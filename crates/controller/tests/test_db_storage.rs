use switchboard_controller::{
    config::ControllerConfig,
    storage::{ListObjectQuery, ObjectFilter},
};
use switchboard_custom_config::switchboard_serde_value::value;
use switchboard_model::PageQuery;

#[tokio::test]
async fn test_db_storage() -> switchboard_controller::Result<()> {
    use switchboard_controller::ControllerContext;
    use switchboard_controller::storage::StorageProvider;
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
    // test get object
    let object_a2 = context.storage().get_object(&descriptor_a2).await?;
    assert_eq!(
        object_a2,
        Some(value!({
            "version": 2,
        }))
    );
    let object_b1 = context.storage().get_object(&descriptor_b1).await?;
    assert_eq!(
        object_b1,
        Some(value!({
            "version": 1,
        }))
    );

    // test delete object
    let deleted_a2 = context
        .storage()
        .delete_object(&descriptor_a2)
        .await?
        .unwrap();
    // try get deleted object
    let get_deleted_a2 = context.storage().get_object(&deleted_a2).await?;
    assert!(get_deleted_a2.is_none());

    // restore deleted object
    let _restored_a2 = context
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
    // try delete objects by id
    context
        .storage()
        .delete_all_objects_by_id("test-object-a")
        .await?;
    let objects_a_list = context
        .storage()
        .list_objects(ListObjectQuery {
            filter: ObjectFilter {
                id: Some("test-object-a".into()),
                ..Default::default()
            },
            page: PageQuery::with_limit(10),
        })
        .await?;
    assert_eq!(objects_a_list.items.len(), 0);
    // try batch delete objects
    let to_delete_1 = context
        .storage()
        .save_object(
            "test-object-to-delete-1",
            DATA_TYPE,
            value!( {
                "version": 1,
            }),
        )
        .await
        .unwrap();
    let to_delete_2 = context
        .storage()
        .save_object(
            "test-object-to-delete-2",
            DATA_TYPE,
            value!( {
                "version": 1,
            }),
        )
        .await
        .unwrap();
    context
        .storage()
        .batch_delete_objects(vec![to_delete_1.clone(), to_delete_2.clone()])
        .await?;
    let check_deleted_1 = context.storage().get_object(&to_delete_1).await?;
    assert!(check_deleted_1.is_none());
    let check_deleted_2 = context.storage().get_object(&to_delete_2).await?;
    assert!(check_deleted_2.is_none());
    Ok(())
}
