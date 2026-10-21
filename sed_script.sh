sed -i '6520i \
    #[test]\
    fn plan_to_json_pretty() {\
        let plan = EditPlan {\
            schema_version: 1,\
            source_sha256: Some("sha".into()),\
            author: "Author".into(),\
            date: Some("2024-01-01T00:00:00Z".into()),\
            initials: Some("A".into()),\
            resolve_revisions: None,\
            existing_revisions: ExistingRevisions::Refuse,\
            removed_anchor_comments: crate::edit::RemovedAnchorComments::default(),\
            operations: vec![],\
            update_fields: false,\
        };\
        \
        let expected_json = r#"{\
  "schema_version": 1,\
  "source_sha256": "sha",\
  "author": "Author",\
  "date": "2024-01-01T00:00:00Z",\
  "initials": "A",\
  "existing_revisions": "refuse",\
  "removed_anchor_comments": "remove",\
  "operations": []\
}"#;\
        assert_eq!(plan.to_json(), expected_json);\
    }' src/edit.rs
