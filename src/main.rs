use std::path::Path;

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

    println!("Target Path: {}", target_path.display());

    let target_folder = StorageFolder::GetFolderFromPathAsync(&HSTRING::from(
        target_path.to_string_lossy().to_string(),
    ))?
    .join()?;

    let qo = QueryOptions::CreateCommonFolderQuery(CommonFolderQuery::DefaultQuery)?;
    qo.SetFolderDepth(FolderDepth::Shallow)?;

    let keyword = SystemProperties::Keywords()?;
    let props = IIterable::<HSTRING>::from(vec![keyword]);

    qo.SetPropertyPrefetch(PropertyPrefetchOptions::BasicProperties, &props)?;

    println!("Query Setup is completed");

    let item_query = target_folder.CreateItemQueryWithOptions(&qo)?;

    println!("Item Query created");

    let items = item_query.GetItemsAsyncDefaultStartAndCount()?.join()?;

    println!("Found {} items.", items.Size()?);

    for item in &items {
        let names: [HSTRING; 1] = [SystemProperties::Keywords()?];
        let properties = item.GetBasicPropertiesAsync()?.join()?;
        let map = properties.RetrievePropertiesAsync(&props)?.join()?;
        if let Some(value) = map.Lookup(&names[0]).ok() {
            println!(
                "{}\ttags:{:?}\tsize:{}\tmodified:{:?}",
                item.Name()?.to_string_lossy(),
                value,
                properties.Size()?,
                properties.DateModified()?
            );
        } else {
            println!(
                "{}\ttags:<no keywords>\tsize:{}\tmodified:{:?}",
                item.Name()?.to_string_lossy(),
                properties.Size()?,
                properties.DateModified()?
            );
        }
    }

    Ok(())
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
