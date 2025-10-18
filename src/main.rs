use std::{collections::VecDeque, path::Path};

use windows::{
    Foundation::{IPropertyValue, PropertyType},
    Storage::{
        FileProperties::PropertyPrefetchOptions,
        Search::{CommonFileQuery, CommonFolderQuery, FolderDepth, QueryOptions},
        StorageFolder, SystemProperties,
    },
    core::{Array, HSTRING, IInspectable, Interface, Result},
};
use windows_collections::{IIterable, IVectorView};

fn main() -> windows::core::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Invalid arguments.");
    }

    let target_path: &Path = args[1].as_ref();

    if !target_path.exists() || !target_path.is_dir() {
        println!(
            "The target path could not be found or is not a directory: {}",
            target_path.display()
        );
        return Ok(());
    }

    println!("Scan root: {}", target_path.display());

    let mut q = VecDeque::<(StorageFolder, u32)>::new();

    let target_folder = StorageFolder::GetFolderFromPathAsync(&HSTRING::from(
        target_path.to_string_lossy().to_string(),
    ))?
    .join()?;

    q.push_back((target_folder, 0));

    while let Some((folder, depth)) = q.pop_front() {
        // println!("Folder: {}", folder.Path()?.to_string_lossy());
        println!(
            "{} {}",
            "  ".repeat(depth as usize),
            folder.Name()?.to_string_lossy()
        );
        let files = list_file(folder.clone())?;
        files.iter().for_each(|file_info| {
            println!(
                "{}- {}\tsize:{}\tmodified:{:?}\tkeywords:{:?}",
                "  ".repeat((depth + 1) as usize),
                file_info.name,
                file_info.size,
                file_info.modified,
                file_info.keywords
            );
        });

        if depth < 2 {
            let subfolders = list_folder(folder.clone())?;
            subfolders.into_iter().for_each(|subfolder| {
                q.push_back((subfolder, depth + 1));
            });
        }
    }

    Ok(())
}

struct FileInfo {
    name: String,
    size: u64,
    modified: windows::Foundation::DateTime,
    keywords: Option<Vec<String>>,
}

fn list_file(folder: StorageFolder) -> Result<Vec<FileInfo>> {
    let qo = build_file_query_options()?;
    let item_query = folder.CreateItemQueryWithOptions(&qo)?;

    let items = item_query.GetItemsAsyncDefaultStartAndCount()?.join()?;

    let mut file_infos = Vec::<FileInfo>::new();

    for item in &items {
        let properties = item.GetBasicPropertiesAsync()?.join()?;

        let names: [HSTRING; 1] = [SystemProperties::Keywords()?];
        let keyword = SystemProperties::Keywords()?;
        let props = IIterable::<HSTRING>::from(vec![keyword]);
        let map = properties.RetrievePropertiesAsync(&props)?.join()?;
        let keywords = if let Some(v) = map.Lookup(&names[0]).ok() {
            Some(to_keywords(&v)?)
        } else {
            None
        };

        let file_info = FileInfo {
            name: item.Name()?.to_string_lossy().to_string(),
            size: properties.Size()?,
            modified: properties.DateModified()?,
            keywords,
        };

        file_infos.push(file_info);
    }

    Ok(file_infos)
}

fn to_keywords(value: &IInspectable) -> Result<Vec<String>> {
    // まず IPropertyValue として解釈してみる
    if let Ok(pv) = value.cast::<IPropertyValue>() {
        match pv.Type()? {
            PropertyType::StringArray => {
                let mut arr: Array<HSTRING> = Array::new();
                pv.GetStringArray(&mut arr)?; // ← ここで配列が入る
                let v = arr.iter().map(|s| s.to_string_lossy()).collect();
                return Ok(v);
            }
            PropertyType::String => {
                // 万一、単一文字列で来た場合は 1 要素の配列として扱う
                let s: HSTRING = pv.GetString()?;
                return Ok(vec![s.to_string_lossy()]);
            }
            PropertyType::Empty => {
                // null（未設定）のケース
                return Ok(Vec::new());
            }
            _ => { /* fallthrough: 別の型かもしれない */ }
        }
    }

    // 稀にコレクション型（IVectorView<HSTRING>）で来る API もあるためフォールバック
    if let Ok(view) = value.cast::<IVectorView<HSTRING>>() {
        let mut out = Vec::with_capacity(view.Size()? as usize);
        for i in 0..view.Size()? {
            out.push(view.GetAt(i)?.to_string_lossy());
        }
        return Ok(out);
    }

    // 想定外の型
    Ok(Vec::new())
}

fn list_folder(folder: StorageFolder) -> Result<Vec<StorageFolder>> {
    let qo = build_folder_query_options()?;
    let folder_query = folder.CreateFolderQueryWithOptions(&qo)?;

    let subfolders = folder_query.GetFoldersAsyncDefaultStartAndCount()?.join()?;

    let mut storage_folders = Vec::<StorageFolder>::new();

    for subfolder in &subfolders {
        storage_folders.push(subfolder.clone());
    }

    Ok(storage_folders)
}

fn build_file_query_options() -> Result<QueryOptions> {
    let qo = QueryOptions::CreateCommonFileQuery(
        CommonFileQuery::DefaultQuery,
        &IIterable::<HSTRING>::from(vec![]),
    )?;
    qo.SetFolderDepth(FolderDepth::Shallow)?;
    qo.SetIndexerOption(windows::Storage::Search::IndexerOption::UseIndexerWhenAvailable)?;

    let keyword = SystemProperties::Keywords()?;
    let props = IIterable::<HSTRING>::from(vec![keyword]);
    qo.SetPropertyPrefetch(PropertyPrefetchOptions::BasicProperties, &props)?;

    Ok(qo)
}

fn build_folder_query_options() -> Result<QueryOptions> {
    let qo = QueryOptions::CreateCommonFolderQuery(CommonFolderQuery::DefaultQuery)?;
    qo.SetFolderDepth(FolderDepth::Shallow)?;
    qo.SetIndexerOption(windows::Storage::Search::IndexerOption::UseIndexerWhenAvailable)?;

    Ok(qo)
}
