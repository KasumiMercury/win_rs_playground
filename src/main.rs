use std::{collections::VecDeque, path::Path};

use windows::{
    Storage::{
        FileProperties::PropertyPrefetchOptions,
        Search::{CommonFileQuery, CommonFolderQuery, FolderDepth, QueryOptions},
        StorageFolder, SystemProperties,
    },
    core::{HSTRING, Result},
};
use windows_collections::IIterable;

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
                "{}- {}\tsize:{}\tmodified:{:?}",
                "  ".repeat((depth + 1) as usize),
                file_info.name,
                file_info.size,
                file_info.modified
            );
        });

        if depth < 2 {
            let subfolders = list_folder(folder.clone())?;
            subfolders.into_iter().for_each(|subfolder| {
                q.push_back((subfolder, depth + 1));
            });
        }
    }

    // let qo = QueryOptions::CreateCommonFolderQuery(CommonFolderQuery::DefaultQuery)?;
    // qo.SetFolderDepth(FolderDepth::Shallow)?;

    // let keyword = SystemProperties::Keywords()?;
    // let props = IIterable::<HSTRING>::from(vec![keyword]);

    // qo.SetPropertyPrefetch(PropertyPrefetchOptions::BasicProperties, &props)?;

    // println!("Query Setup is completed");

    // let item_query = target_folder.CreateItemQueryWithOptions(&qo)?;

    // println!("Item Query created");

    // let items = item_query.GetItemsAsyncDefaultStartAndCount()?.join()?;

    // println!("Found {} items.", items.Size()?);

    // for item in &items {
    //     let names: [HSTRING; 1] = [SystemProperties::Keywords()?];
    //     let properties = item.GetBasicPropertiesAsync()?.join()?;
    //     let map = properties.RetrievePropertiesAsync(&props)?.join()?;
    //     if let Some(value) = map.Lookup(&names[0]).ok() {
    //         println!(
    //             "{}\ttags:{:?}\tsize:{}\tmodified:{:?}",
    //             item.Name()?.to_string_lossy(),
    //             value,
    //             properties.Size()?,
    //             properties.DateModified()?
    //         );
    //     } else {
    //         println!(
    //             "{}\ttags:<no keywords>\tsize:{}\tmodified:{:?}",
    //             item.Name()?.to_string_lossy(),
    //             properties.Size()?,
    //             properties.DateModified()?
    //         );
    //     }
    // }

    Ok(())
}

struct FileInfo {
    name: String,
    size: u64,
    modified: windows::Foundation::DateTime,
}

fn list_file(folder: StorageFolder) -> Result<Vec<FileInfo>> {
    let qo = build_file_query_options()?;
    let item_query = folder.CreateItemQueryWithOptions(&qo)?;

    let items = item_query.GetItemsAsyncDefaultStartAndCount()?.join()?;

    let mut file_infos = Vec::<FileInfo>::new();

    for item in &items {
        let properties = item.GetBasicPropertiesAsync()?.join()?;
        let file_info = FileInfo {
            name: item.Name()?.to_string_lossy().to_string(),
            size: properties.Size()?,
            modified: properties.DateModified()?,
        };

        file_infos.push(file_info);
    }

    Ok(file_infos)
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
