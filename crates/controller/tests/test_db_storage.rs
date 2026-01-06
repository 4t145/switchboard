use switchboard_controller::{
    config::ControllerConfig,
    storage::{ListObjectQuery, ObjectFilter},
};
use switchboard_custom_config::switchboard_serde_value::value;
use switchboard_model::PageQuery;

#[tokio::test]
async fn test_db_storage() -> switchboard_controller::Result<()> {
    use switchboard_controller::ControllerContext;
    use switchboard_controller::storage::{StorageObjectDescriptor, StorageProvider};
    const DATA_TYPE: &str = "test-data";
    const DB_PATH: &str = "tmp/test_db_storage.db";
    if tokio::fs::try_exists(DB_PATH).await.unwrap() {
        tokio::fs::remove_dir_all(DB_PATH).await.unwrap();
    }
    // check if
    let context = ControllerContext::new(ControllerConfig {
        storage: StorageProvider::Local {
            db_file: DB_PATH.into(),
        },
        ..Default::default()
    })
    .await?;

    // test save object
    let descriptor_a1 = context
        .storage()
        .save_object(
            "test-object-a",
            DATA_TYPE,
            value!( {
                "version": 1,
            }),
        )
        .await
        .unwrap();

    assert_eq!(descriptor_a1.id, "test-object-a");

    // test save object
    let descriptor_a2 = context
        .storage()
        .save_object(
            "test-object-a",
            DATA_TYPE,
            value!( {
                "version": 2,
            }),
        )
        .await
        .unwrap();

    assert_eq!(descriptor_a2.id, "test-object-a");

    let descriptor_b1 = context
        .storage()
        .save_object(
            "test-object-b",
            DATA_TYPE,
            value!( {
                "version": 1,
            }),
        )
        .await
        .unwrap();

    assert_eq!(descriptor_b1.id, "test-object-b");

    // test list objects
    let objects = context
        .storage()
        .list_objects(ListObjectQuery {
            filter: ObjectFilter {
                data_type: Some(DATA_TYPE.into()),
                ..Default::default()
            },
            page: PageQuery::with_limit(10),
        })
        .await
        .unwrap();
    assert_eq!(objects.items.len(), 3);

    // test list latest objects
    let first_page = context
        .storage()
        .list_objects(ListObjectQuery {
            filter: ObjectFilter {
                data_type: Some(DATA_TYPE.into()),
                latest_only: Some(true),
                ..Default::default()
            },
            page: PageQuery::with_limit(10),
        })
        .await
        .unwrap();
    println!("objects: {:#?}", first_page);
    assert_eq!(first_page.items.len(), 2);
    let next_cursor = first_page.next_cursor.unwrap();
    let second_page = context
        .storage()
        .list_objects(ListObjectQuery {
            filter: ObjectFilter {
                data_type: Some(DATA_TYPE.into()),
                latest_only: Some(true),
                ..Default::default()
            },
            page: PageQuery::with_limit(10).with_cursor(next_cursor),
        })
        .await
        .unwrap();
    println!("objects: {:#?}", second_page);
    assert_eq!(second_page.items.len(), 0);
    // test
    Ok(())
}
