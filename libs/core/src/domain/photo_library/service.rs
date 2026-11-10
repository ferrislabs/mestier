use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Doe, DoeFile, DoeSectionContent, LibraryPhoto, LibraryPhotoId, OrganizationId, PhotoFilter,
    Project, ProjectId,
    domain::{
        doe::{
            ports::DoeRepository,
            service::{FileTarget, place_file},
        },
        file_storage::{
            commands::UploadFileCommand, ports::FileStorage, service::FileStorageService,
        },
        photo_library::{
            BackfillReport, InsertOutcome, PhotoLibraryError, PhotoLibraryOutcome, PlacementReport,
            PlacementResult, PlacementStatus, UploadedPhoto,
            commands::{
                ListLibraryPhotosCommand, MAX_PLACEMENTS, PlaceLibraryPhotosCommand,
                PlacementRequest, PlacementTargetKind, UpdateLibraryPhotoCommand,
                UploadLibraryPhotoCommand,
            },
            normalize::{NormalizedImage, STORED_MIME_TYPE, hex_digest, normalize_image},
            ports::LibraryPhotoRepository,
            tags::{normalize_caption, normalize_tag, normalize_tags},
        },
        project::ports::ProjectRepository,
    },
};

/// Every photo a DOE prints: the cover, the photo sections, the supply sheets.
/// Plans and datasheets are documents, not photos. A key listed twice is read
/// once, with the first legend.
fn doe_photos(doe: &Doe) -> Vec<DoeFile> {
    let mut files: Vec<DoeFile> = Vec::new();
    let mut push = |file: DoeFile| {
        if !files.iter().any(|known| known.key == file.key) {
            files.push(file);
        }
    };
    if let Some(key) = &doe.cover_photo_key {
        push(DoeFile {
            key: key.clone(),
            caption: None,
        });
    }
    for section in &doe.sections {
        match &section.content {
            DoeSectionContent::Photos { photos, .. } => photos.iter().cloned().for_each(&mut push),
            DoeSectionContent::Supplies { supplies, .. } => supplies
                .iter()
                .flat_map(|supply| supply.photos.iter().cloned())
                .for_each(&mut push),
            DoeSectionContent::Plans { .. } | DoeSectionContent::BlankPages { .. } => {}
        }
    }
    files
}

const DOE_FILE_FOLDER: &str = "doe";
const STYLE_FILE_FOLDER: &str = "doe-style";

fn library_folder(organization_id: OrganizationId) -> String {
    format!("library-{}", organization_id.0)
}

async fn require_project<PR: ProjectRepository>(
    projects: &mut PR,
    organization_id: OrganizationId,
    project_id: ProjectId,
) -> Result<(), CoreError> {
    projects
        .find_by_id(project_id)
        .await?
        .filter(|project: &Project| project.organization_id == organization_id)
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

async fn discard<S: FileStorage>(storage: &FileStorageService<S>, keys: &[&str]) {
    for key in keys {
        if let Err(error) = storage.delete(key).await {
            tracing::warn!(key, error = %error, "could not remove an orphaned library object");
        }
    }
}

pub struct PhotoLibraryService<LR>
where
    LR: LibraryPhotoRepository,
{
    photos: LR,
}

impl<LR> PhotoLibraryService<LR>
where
    LR: LibraryPhotoRepository,
{
    pub fn new(photos: LR) -> Self {
        Self { photos }
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
    ) -> Result<LibraryPhoto, CoreError> {
        self.photos
            .find_by_id(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    /// Copies a library photo to a key of its own, so that deleting the photo
    /// never takes the copy with it.
    pub async fn copy_file<S: FileStorage>(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
        storage: &FileStorageService<S>,
    ) -> Result<String, CoreError> {
        let photo = self.get(organization_id, id).await?;
        let object = storage.get(&photo.file_key).await?;
        let stored = storage
            .upload(UploadFileCommand {
                mime_type: object.mime_type,
                bytes: object.bytes,
                folder: Some(STYLE_FILE_FOLDER.to_owned()),
            })
            .await?;
        Ok(stored.key)
    }

    pub async fn list(
        &mut self,
        command: ListLibraryPhotosCommand,
    ) -> Result<PhotoLibraryOutcome<(Vec<LibraryPhoto>, u64)>, CoreError> {
        let mut tags = Vec::with_capacity(command.tags.len());
        for tag in &command.tags {
            match normalize_tag(tag) {
                Ok(tag) => tags.push(tag),
                Err(error) => return Ok(Err(error)),
            }
        }
        let filter = PhotoFilter {
            query: command
                .query
                .as_deref()
                .map(str::trim)
                .filter(|query| !query.is_empty())
                .map(str::to_owned),
            tags,
            project_id: command.project_id,
            to_sort: command.to_sort,
        };

        self.photos
            .list(
                command.organization_id,
                &filter,
                command.limit,
                command.offset,
            )
            .await
            .map(Ok)
    }

    pub async fn upload<PR, S>(
        &mut self,
        command: UploadLibraryPhotoCommand,
        projects: &mut PR,
        storage: &FileStorageService<S>,
    ) -> Result<UploadedPhoto, CoreError>
    where
        PR: ProjectRepository,
        S: FileStorage,
    {
        let UploadLibraryPhotoCommand {
            organization_id,
            project_id,
            image,
            ..
        } = command;

        if let Some(project_id) = project_id {
            require_project(projects, organization_id, project_id).await?;
        }

        self.store_image(organization_id, project_id, None, image, storage)
            .await
    }

    async fn store_image<S>(
        &mut self,
        organization_id: OrganizationId,
        project_id: Option<ProjectId>,
        caption: Option<String>,
        image: NormalizedImage,
        storage: &FileStorageService<S>,
    ) -> Result<UploadedPhoto, CoreError>
    where
        S: FileStorage,
    {
        if let Some(photo) = self
            .photos
            .find_by_hash(organization_id, &image.content_hash)
            .await?
        {
            return Ok(UploadedPhoto {
                photo,
                created: false,
            });
        }

        let size_bytes = u32::try_from(image.bytes.len())
            .map_err(|_| CoreError::Internal("normalized image is too large".to_owned()))?;
        let folder = library_folder(organization_id);

        let file = storage
            .upload(UploadFileCommand {
                mime_type: STORED_MIME_TYPE.to_owned(),
                bytes: image.bytes,
                folder: Some(folder.clone()),
            })
            .await?;
        let thumbnail = match storage
            .upload(UploadFileCommand {
                mime_type: STORED_MIME_TYPE.to_owned(),
                bytes: image.thumbnail,
                folder: Some(folder),
            })
            .await
        {
            Ok(thumbnail) => thumbnail,
            Err(error) => {
                discard(storage, &[&file.key]).await;
                return Err(error);
            }
        };

        let now = Utc::now();
        let photo = LibraryPhoto {
            id: LibraryPhotoId(generate_uuid_v7()),
            organization_id,
            file_key: file.key,
            thumbnail_key: thumbnail.key,
            mime_type: STORED_MIME_TYPE.to_owned(),
            width: image.width,
            height: image.height,
            size_bytes,
            content_hash: image.content_hash,
            caption,
            tags: Vec::new(),
            project_id,
            taken_at: image.taken_at,
            created_at: now,
            updated_at: now,
        };

        match self.photos.insert_if_absent(&photo).await {
            Ok(InsertOutcome::Inserted(photo)) => Ok(UploadedPhoto {
                photo,
                created: true,
            }),
            Ok(InsertOutcome::Existing(existing)) => {
                discard(storage, &[&photo.file_key, &photo.thumbnail_key]).await;
                Ok(UploadedPhoto {
                    photo: existing,
                    created: false,
                })
            }
            Err(error) => {
                discard(storage, &[&photo.file_key, &photo.thumbnail_key]).await;
                Err(error)
            }
        }
    }

    /// Copies every photo of an existing DOE into the library, keeping its
    /// legend and the project it came from. A photo already in the library
    /// (same normalized content) is left as it is.
    pub async fn backfill_doe<S>(
        &mut self,
        doe: &Doe,
        storage: &FileStorageService<S>,
    ) -> BackfillReport
    where
        S: FileStorage,
    {
        let mut report = BackfillReport::default();
        for file in doe_photos(doe) {
            let object = match storage.get(&file.key).await {
                Ok(object) => object,
                Err(error) => {
                    tracing::warn!(key = %file.key, error = %error, "backfill: cannot read the DOE photo");
                    report.failed += 1;
                    continue;
                }
            };
            // A copy placed from the library is already normalized: re-encoding
            // it would change its hash and import it a second time.
            match self
                .photos
                .find_by_hash(doe.organization_id, &hex_digest(&object.bytes))
                .await
            {
                Ok(Some(_)) => {
                    report.already_present += 1;
                    continue;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(key = %file.key, error = %error, "backfill: cannot look the photo up");
                    report.failed += 1;
                    continue;
                }
            }
            let image = match normalize_image(&object.bytes) {
                Ok(image) => image,
                Err(error) => {
                    tracing::warn!(key = %file.key, error = %error, "backfill: not an importable image");
                    report.skipped += 1;
                    continue;
                }
            };
            let caption = normalize_caption(file.caption.as_deref());
            match self
                .store_image(
                    doe.organization_id,
                    Some(doe.project_id),
                    caption,
                    image,
                    storage,
                )
                .await
            {
                Ok(uploaded) if uploaded.created => report.imported += 1,
                Ok(_) => report.already_present += 1,
                Err(error) => {
                    tracing::warn!(key = %file.key, error = %error, "backfill: cannot store the photo");
                    report.failed += 1;
                }
            }
        }
        report
    }

    pub async fn update<PR>(
        &mut self,
        command: UpdateLibraryPhotoCommand,
        projects: &mut PR,
    ) -> Result<PhotoLibraryOutcome<LibraryPhoto>, CoreError>
    where
        PR: ProjectRepository,
    {
        let mut photo = self.get(command.organization_id, command.id).await?;

        if let Some(tags) = &command.tags {
            match normalize_tags(tags) {
                Ok(tags) => photo.tags = tags,
                Err(error) => return Ok(Err(error)),
            }
        }
        if let Some(caption) = &command.caption {
            photo.caption = normalize_caption(Some(caption));
        }
        if let Some(project_id) = command.project_id {
            if let Some(project_id) = project_id {
                require_project(projects, command.organization_id, project_id).await?;
            }
            photo.project_id = project_id;
        }
        photo.updated_at = Utc::now();

        self.photos.update(&photo).await.map(Ok)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
    ) -> Result<LibraryPhoto, CoreError> {
        let photo = self.get(organization_id, id).await?;
        self.photos.delete(organization_id, id).await?;
        Ok(photo)
    }

    pub async fn place<DR, S>(
        &mut self,
        command: PlaceLibraryPhotosCommand,
        does: &mut DR,
        storage: &FileStorageService<S>,
    ) -> Result<PhotoLibraryOutcome<PlacementReport>, CoreError>
    where
        DR: DoeRepository,
        S: FileStorage,
    {
        if command.placements.len() > MAX_PLACEMENTS {
            return Ok(Err(PhotoLibraryError::TooManyPlacements));
        }

        let doe = does
            .find_by_id(command.doe_id)
            .await?
            .filter(|doe| doe.organization_id == command.organization_id)
            .ok_or(CoreError::NotFound)?;

        let mut working = doe.clone();
        let mut copied: Vec<String> = Vec::new();
        let mut results = Vec::with_capacity(command.placements.len());

        for request in &command.placements {
            let outcome = self
                .place_one(command.organization_id, &mut working, request, storage)
                .await?;
            results.push(match outcome {
                Ok(key) => {
                    copied.push(key.clone());
                    PlacementResult {
                        photo_id: request.photo_id,
                        status: PlacementStatus::Placed,
                        key: Some(key),
                        error: None,
                    }
                }
                Err(error) => PlacementResult {
                    photo_id: request.photo_id,
                    status: PlacementStatus::Failed,
                    key: None,
                    error: Some(error),
                },
            });
        }

        if copied.is_empty() {
            return Ok(Ok(PlacementReport { doe, results }));
        }

        working.updated_at = Utc::now();
        match does.update(&working).await {
            Ok(doe) => Ok(Ok(PlacementReport { doe, results })),
            Err(error) => {
                let keys: Vec<&str> = copied.iter().map(String::as_str).collect();
                discard(storage, &keys).await;
                Err(error)
            }
        }
    }

    async fn place_one<S>(
        &mut self,
        organization_id: OrganizationId,
        working: &mut Doe,
        request: &PlacementRequest,
        storage: &FileStorageService<S>,
    ) -> Result<Result<String, String>, CoreError>
    where
        S: FileStorage,
    {
        let Some(photo) = self
            .photos
            .find_by_id(organization_id, request.photo_id)
            .await?
        else {
            return Ok(Err("photo not found".to_owned()));
        };
        let target = match file_target(request) {
            Ok(target) => target,
            Err(message) => return Ok(Err(message)),
        };
        let caption = match &request.caption {
            Some(caption) => normalize_caption(Some(caption)),
            None => photo.caption.clone(),
        };

        let probe_key = format!("pending-{}", generate_uuid_v7());
        if let Err(refusal) = place_file(working, target, probe_key, caption.clone()) {
            return Ok(Err(refusal.to_string()));
        }

        let object = match storage.get(&photo.file_key).await {
            Ok(object) => object,
            Err(CoreError::NotFound) => {
                return Ok(Err("the photo file is missing from storage".to_owned()));
            }
            Err(error) => {
                tracing::warn!(error = %error, "reading a library photo for placement failed");
                return Ok(Err("the photo could not be read".to_owned()));
            }
        };
        let stored = match storage
            .upload(UploadFileCommand {
                mime_type: object.mime_type,
                bytes: object.bytes,
                folder: Some(DOE_FILE_FOLDER.to_owned()),
            })
            .await
        {
            Ok(stored) => stored,
            Err(error) => {
                tracing::warn!(error = %error, "copying a library photo into a DOE failed");
                return Ok(Err("the photo could not be copied".to_owned()));
            }
        };

        match place_file(working, target, stored.key.clone(), caption) {
            Ok(placed) => {
                *working = placed;
                Ok(Ok(stored.key))
            }
            Err(refusal) => {
                discard(storage, &[&stored.key]).await;
                Ok(Err(refusal.to_string()))
            }
        }
    }
}

fn file_target(request: &PlacementRequest) -> Result<FileTarget<'_>, String> {
    match request.target {
        PlacementTargetKind::Cover => Ok(FileTarget::Cover),
        PlacementTargetKind::Section => Ok(FileTarget::Section {
            id: request.section_id,
            title: request.section_title.as_deref(),
        }),
        PlacementTargetKind::Supply => request
            .supply_id
            .map(|id| FileTarget::Supply { id })
            .ok_or_else(|| "supply_id is required to place a photo in a supply".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::{
        DoeSection, DoeSectionContent, FileObject, PhotosPerPage, PresignedUrl, Revision,
        StoredFile,
        domain::{
            doe::{DoeFile, DoeId, ports::MockDoeRepository},
            file_storage::ports::FileUpload,
            photo_library::{normalize::NormalizedImage, ports::MockLibraryPhotoRepository},
            project::ports::MockProjectRepository,
        },
    };

    #[derive(Clone, Default)]
    struct FakeStorage {
        objects: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    }

    impl FakeStorage {
        fn keys(&self) -> Vec<String> {
            self.objects.lock().unwrap().keys().cloned().collect()
        }
    }

    impl FileStorage for FakeStorage {
        async fn upload(&self, upload: FileUpload) -> Result<StoredFile, CoreError> {
            let size_bytes = upload.bytes.len() as u64;
            self.objects
                .lock()
                .unwrap()
                .insert(upload.key.clone(), upload.bytes);
            Ok(StoredFile {
                key: upload.key,
                mime_type: upload.mime_type,
                size_bytes,
            })
        }

        async fn get(&self, key: &str) -> Result<FileObject, CoreError> {
            let bytes = self
                .objects
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or(CoreError::NotFound)?;
            Ok(FileObject {
                key: key.to_owned(),
                mime_type: "image/jpeg".to_owned(),
                size_bytes: bytes.len() as u64,
                bytes,
            })
        }

        async fn presigned_get_url(
            &self,
            _key: &str,
            _expires_in: std::time::Duration,
        ) -> Result<PresignedUrl, CoreError> {
            Err(CoreError::Internal("not needed".to_owned()))
        }

        async fn delete(&self, key: &str) -> Result<(), CoreError> {
            self.objects.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn storage_service(storage: FakeStorage) -> FileStorageService<FakeStorage> {
        FileStorageService::new(storage, "")
    }

    fn organization() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn image(hash: &str) -> NormalizedImage {
        NormalizedImage {
            bytes: vec![1, 2, 3],
            thumbnail: vec![4, 5],
            width: 30,
            height: 20,
            content_hash: hash.to_owned(),
            taken_at: None,
        }
    }

    fn photo(organization_id: OrganizationId, file_key: &str) -> LibraryPhoto {
        let now = Utc::now();
        LibraryPhoto {
            id: LibraryPhotoId(Uuid::new_v4()),
            organization_id,
            file_key: file_key.to_owned(),
            thumbnail_key: format!("{file_key}.thumb"),
            mime_type: "image/jpeg".to_owned(),
            width: 30,
            height: 20,
            size_bytes: 3,
            content_hash: Uuid::new_v4().to_string(),
            caption: Some("Massif de rosiers".to_owned()),
            tags: Vec::new(),
            project_id: None,
            taken_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn upload_command(organization_id: OrganizationId, hash: &str) -> UploadLibraryPhotoCommand {
        UploadLibraryPhotoCommand {
            actor: authz::Subject::system(),
            organization_id,
            project_id: None,
            image: image(hash),
        }
    }

    fn inserting() -> MockLibraryPhotoRepository {
        let mut repository = MockLibraryPhotoRepository::new();
        repository
            .expect_find_by_hash()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repository.expect_insert_if_absent().returning(|photo| {
            let photo = photo.clone();
            Box::pin(async move { Ok(InsertOutcome::Inserted(photo)) })
        });
        repository
    }

    #[tokio::test]
    async fn a_new_image_is_stored_twice_and_recorded_once() {
        let storage = FakeStorage::default();
        let organization_id = organization();

        let uploaded = PhotoLibraryService::new(inserting())
            .upload(
                upload_command(organization_id, "abc"),
                &mut MockProjectRepository::new(),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap();

        assert!(uploaded.created);
        assert_eq!(storage.keys().len(), 2);
        assert!(storage.keys().contains(&uploaded.photo.file_key));
        assert!(storage.keys().contains(&uploaded.photo.thumbnail_key));
        assert!(
            uploaded
                .photo
                .file_key
                .starts_with(&format!("library-{}/", organization_id.0))
        );
        assert_eq!(uploaded.photo.mime_type, "image/jpeg");
        assert_eq!(uploaded.photo.size_bytes, 3);
        assert_eq!(uploaded.photo.caption, None);
        assert!(uploaded.photo.tags.is_empty());
    }

    #[tokio::test]
    async fn an_image_the_organization_already_holds_stores_nothing() {
        let organization_id = organization();
        let existing = photo(organization_id, "library/x.jpg");
        let returned = existing.clone();
        let mut repository = MockLibraryPhotoRepository::new();
        repository.expect_find_by_hash().returning(move |_, _| {
            let existing = returned.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });
        let storage = FakeStorage::default();

        let uploaded = PhotoLibraryService::new(repository)
            .upload(
                upload_command(organization_id, "abc"),
                &mut MockProjectRepository::new(),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap();

        assert!(!uploaded.created);
        assert_eq!(uploaded.photo, existing);
        assert!(storage.keys().is_empty());
    }

    #[tokio::test]
    async fn losing_the_insert_race_removes_the_objects_just_written() {
        let organization_id = organization();
        let winner = photo(organization_id, "library/winner.jpg");
        let returned = winner.clone();
        let mut repository = MockLibraryPhotoRepository::new();
        repository
            .expect_find_by_hash()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repository.expect_insert_if_absent().returning(move |_| {
            let winner = returned.clone();
            Box::pin(async move { Ok(InsertOutcome::Existing(winner)) })
        });
        let storage = FakeStorage::default();

        let uploaded = PhotoLibraryService::new(repository)
            .upload(
                upload_command(organization_id, "abc"),
                &mut MockProjectRepository::new(),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap();

        assert!(!uploaded.created);
        assert_eq!(uploaded.photo, winner);
        assert!(storage.keys().is_empty());
    }

    #[tokio::test]
    async fn a_failed_insert_removes_the_objects_just_written() {
        let mut repository = MockLibraryPhotoRepository::new();
        repository
            .expect_find_by_hash()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repository
            .expect_insert_if_absent()
            .returning(|_| Box::pin(async { Err(CoreError::Database("down".to_owned())) }));
        let storage = FakeStorage::default();

        let result = PhotoLibraryService::new(repository)
            .upload(
                upload_command(organization(), "abc"),
                &mut MockProjectRepository::new(),
                &storage_service(storage.clone()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Database(_))));
        assert!(storage.keys().is_empty());
    }

    #[tokio::test]
    async fn a_project_of_another_organization_is_refused_before_anything_is_stored() {
        let organization_id = organization();
        let mut projects = MockProjectRepository::new();
        projects.expect_find_by_id().returning(|_| {
            Box::pin(async {
                let now = Utc::now();
                Ok(Some(crate::Project {
                    id: ProjectId(Uuid::new_v4()),
                    organization_id: OrganizationId(Uuid::new_v4()),
                    name: "Chantier".to_owned(),
                    customer_id: None,
                    customer_context_id: None,
                    quote_id: None,
                    archived_at: None,
                    created_at: now,
                    updated_at: now,
                }))
            })
        });
        let storage = FakeStorage::default();
        let mut command = upload_command(organization_id, "abc");
        command.project_id = Some(ProjectId(Uuid::new_v4()));

        let result = PhotoLibraryService::new(MockLibraryPhotoRepository::new())
            .upload(command, &mut projects, &storage_service(storage.clone()))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
        assert!(storage.keys().is_empty());
    }

    fn repository_holding(photos: Vec<LibraryPhoto>) -> MockLibraryPhotoRepository {
        let mut repository = MockLibraryPhotoRepository::new();
        repository.expect_find_by_id().returning(move |org, id| {
            let found = photos
                .iter()
                .find(|photo| photo.id == id && photo.organization_id == org)
                .cloned();
            Box::pin(async move { Ok(found) })
        });
        repository
    }

    #[tokio::test]
    async fn another_organizations_photo_is_not_found() {
        let photo = photo(organization(), "library/a.jpg");
        let id = photo.id;

        let result = PhotoLibraryService::new(repository_holding(vec![photo]))
            .get(organization(), id)
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    fn update_command(photo: &LibraryPhoto) -> UpdateLibraryPhotoCommand {
        UpdateLibraryPhotoCommand {
            actor: authz::Subject::system(),
            organization_id: photo.organization_id,
            id: photo.id,
            caption: None,
            tags: None,
            project_id: None,
        }
    }

    fn updating(photos: Vec<LibraryPhoto>) -> MockLibraryPhotoRepository {
        let mut repository = repository_holding(photos);
        repository.expect_update().returning(|photo| {
            let photo = photo.clone();
            Box::pin(async move { Ok(photo) })
        });
        repository
    }

    #[tokio::test]
    async fn an_update_normalizes_tags_and_clears_a_blank_caption() {
        let photo = photo(organization(), "library/a.jpg");
        let mut command = update_command(&photo);
        command.caption = Some("   ".to_owned());
        command.tags = Some(vec![
            " Rosier ".to_owned(),
            "ROSIER".to_owned(),
            "Taille".to_owned(),
        ]);

        let updated = PhotoLibraryService::new(updating(vec![photo]))
            .update(command, &mut MockProjectRepository::new())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.caption, None);
        assert_eq!(updated.tags, vec!["rosier", "taille"]);
    }

    #[tokio::test]
    async fn absent_fields_are_left_unchanged() {
        let mut photo = photo(organization(), "library/a.jpg");
        photo.tags = vec!["rosier".to_owned()];
        let command = update_command(&photo);

        let updated = PhotoLibraryService::new(updating(vec![photo.clone()]))
            .update(command, &mut MockProjectRepository::new())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.caption, photo.caption);
        assert_eq!(updated.tags, photo.tags);
    }

    #[tokio::test]
    async fn an_invalid_tag_is_an_outcome_and_writes_nothing() {
        let photo = photo(organization(), "library/a.jpg");
        let mut command = update_command(&photo);
        command.tags = Some(vec![String::new()]);

        let outcome = PhotoLibraryService::new(repository_holding(vec![photo]))
            .update(command, &mut MockProjectRepository::new())
            .await
            .unwrap();

        assert_eq!(outcome, Err(PhotoLibraryError::TagEmpty));
    }

    #[tokio::test]
    async fn a_project_can_be_cleared() {
        let mut photo = photo(organization(), "library/a.jpg");
        photo.project_id = Some(ProjectId(Uuid::new_v4()));
        let mut command = update_command(&photo);
        command.project_id = Some(None);

        let updated = PhotoLibraryService::new(updating(vec![photo]))
            .update(command, &mut MockProjectRepository::new())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.project_id, None);
    }

    #[tokio::test]
    async fn a_list_filter_is_normalized() {
        let mut repository = MockLibraryPhotoRepository::new();
        repository.expect_list().returning(|_, filter, _, _| {
            assert_eq!(filter.tags, vec!["rosier"]);
            assert_eq!(filter.query.as_deref(), Some("50% de_rose"));
            assert!(filter.to_sort);
            Box::pin(async { Ok((Vec::new(), 0)) })
        });

        PhotoLibraryService::new(repository)
            .list(ListLibraryPhotosCommand {
                actor: authz::Subject::system(),
                organization_id: organization(),
                query: Some("  50% de_rose ".to_owned()),
                tags: vec![" Rosier".to_owned()],
                project_id: None,
                to_sort: true,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap()
            .unwrap();
    }

    fn doe_in(organization_id: OrganizationId, sections: Vec<DoeSection>) -> Doe {
        let now = Utc::now();
        Doe {
            id: DoeId(Uuid::new_v4()),
            organization_id,
            project_id: ProjectId(Uuid::new_v4()),
            model_id: None,
            title: "Avenue".to_owned(),
            place: None,
            lot: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
            author: None,
            revision: Revision::try_from("A".to_owned()).unwrap(),
            cover_photo_key: None,
            parties: Vec::new(),
            sections,
            created_at: now,
            updated_at: now,
        }
    }

    fn does_holding(doe: Doe, writes: usize) -> MockDoeRepository {
        let mut repository = MockDoeRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let doe = doe.clone();
            Box::pin(async move { Ok(Some(doe)) })
        });
        repository.expect_update().times(writes).returning(|doe| {
            let doe = doe.clone();
            Box::pin(async move { Ok(doe) })
        });
        repository
    }

    fn request(photo: &LibraryPhoto, target: PlacementTargetKind) -> PlacementRequest {
        PlacementRequest {
            photo_id: photo.id,
            target,
            section_id: None,
            section_title: None,
            supply_id: None,
            caption: None,
        }
    }

    fn place_command(
        organization_id: OrganizationId,
        doe: &Doe,
        placements: Vec<PlacementRequest>,
    ) -> PlaceLibraryPhotosCommand {
        PlaceLibraryPhotosCommand {
            actor: authz::Subject::system(),
            organization_id,
            doe_id: doe.id,
            placements,
        }
    }

    fn photos_in(doe: &Doe) -> Vec<&DoeFile> {
        doe.sections
            .iter()
            .filter_map(|section| match &section.content {
                DoeSectionContent::Photos { photos, .. } => Some(photos.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn library_with(
        organization_id: OrganizationId,
        count: usize,
    ) -> (Vec<LibraryPhoto>, FakeStorage) {
        let storage = FakeStorage::default();
        let photos: Vec<LibraryPhoto> = (0..count)
            .map(|n| photo(organization_id, &format!("library-x/{n}.jpg")))
            .collect();
        for photo in &photos {
            storage
                .objects
                .lock()
                .unwrap()
                .insert(photo.file_key.clone(), vec![9, 9, 9]);
        }
        (photos, storage)
    }

    #[tokio::test]
    async fn three_photos_create_the_general_section_and_are_copied() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let (photos, storage) = library_with(organization_id, 3);
        let placements = photos
            .iter()
            .map(|photo| request(photo, PlacementTargetKind::Section))
            .collect();

        let report = PhotoLibraryService::new(repository_holding(photos.clone()))
            .place(
                place_command(organization_id, &doe, placements),
                &mut does_holding(doe.clone(), 1),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.doe.sections.len(), 1);
        assert_eq!(report.doe.sections[0].title, "Photos générales du chantier");
        let placed = photos_in(&report.doe);
        assert_eq!(placed.len(), 3);
        assert!(
            placed
                .iter()
                .all(|file| photos.iter().all(|photo| photo.file_key != file.key))
        );
        assert!(
            report
                .results
                .iter()
                .all(|result| { result.status == PlacementStatus::Placed && result.key.is_some() })
        );
        assert_eq!(storage.keys().len(), 6);
        assert!(placed.iter().all(|file| file.key.starts_with("doe/")));
        assert_eq!(placed[0].caption.as_deref(), Some("Massif de rosiers"));
    }

    #[tokio::test]
    async fn an_invalid_element_is_reported_and_the_valid_ones_are_written_once() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let (photos, storage) = library_with(organization_id, 2);
        let stranger = photo(organization(), "library-y/stranger.jpg");
        let placements = vec![
            request(&photos[0], PlacementTargetKind::Section),
            request(&stranger, PlacementTargetKind::Section),
            request(&photos[1], PlacementTargetKind::Section),
        ];

        let report = PhotoLibraryService::new(repository_holding(
            photos.iter().cloned().chain([stranger]).collect(),
        ))
        .place(
            place_command(organization_id, &doe, placements),
            &mut does_holding(doe.clone(), 1),
            &storage_service(storage.clone()),
        )
        .await
        .unwrap()
        .unwrap();

        let statuses: Vec<PlacementStatus> =
            report.results.iter().map(|result| result.status).collect();
        assert_eq!(
            statuses,
            vec![
                PlacementStatus::Placed,
                PlacementStatus::Failed,
                PlacementStatus::Placed
            ]
        );
        assert!(report.results[1].error.is_some());
        assert_eq!(photos_in(&report.doe).len(), 2);
        assert_eq!(storage.keys().len(), 4);
    }

    #[tokio::test]
    async fn nothing_is_written_when_no_placement_is_valid() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let ghost = photo(organization_id, "library-x/ghost.jpg");
        let storage = FakeStorage::default();

        let report = PhotoLibraryService::new(repository_holding(Vec::new()))
            .place(
                place_command(
                    organization_id,
                    &doe,
                    vec![request(&ghost, PlacementTargetKind::Cover)],
                ),
                &mut does_holding(doe.clone(), 0),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.doe, doe);
        assert_eq!(report.results[0].status, PlacementStatus::Failed);
        assert!(storage.keys().is_empty());
    }

    #[tokio::test]
    async fn placements_beyond_the_file_ceiling_fail_without_copying() {
        let organization_id = organization();
        let existing: Vec<DoeFile> = (0..299)
            .map(|n| DoeFile {
                key: format!("doe/{n}"),
                caption: None,
            })
            .collect();
        let doe = doe_in(
            organization_id,
            vec![DoeSection {
                id: Uuid::new_v4(),
                title: "Photos".to_owned(),
                enabled: true,
                content: DoeSectionContent::Photos {
                    per_page: PhotosPerPage::Two,
                    photos: existing,
                },
            }],
        );
        let (photos, storage) = library_with(organization_id, 3);
        let placements = photos
            .iter()
            .map(|photo| request(photo, PlacementTargetKind::Section))
            .collect();

        let report = PhotoLibraryService::new(repository_holding(photos))
            .place(
                place_command(organization_id, &doe, placements),
                &mut does_holding(doe.clone(), 1),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap()
            .unwrap();

        let statuses: Vec<PlacementStatus> =
            report.results.iter().map(|result| result.status).collect();
        assert_eq!(
            statuses,
            vec![
                PlacementStatus::Placed,
                PlacementStatus::Failed,
                PlacementStatus::Failed
            ]
        );
        assert_eq!(photos_in(&report.doe).len(), 300);
        assert_eq!(storage.keys().len(), 4);
    }

    #[tokio::test]
    async fn the_cover_target_replaces_the_cover() {
        let organization_id = organization();
        let mut doe = doe_in(organization_id, Vec::new());
        doe.cover_photo_key = Some("doe/old".to_owned());
        let (photos, storage) = library_with(organization_id, 1);

        let report = PhotoLibraryService::new(repository_holding(photos.clone()))
            .place(
                place_command(
                    organization_id,
                    &doe,
                    vec![request(&photos[0], PlacementTargetKind::Cover)],
                ),
                &mut does_holding(doe.clone(), 1),
                &storage_service(storage),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.doe.cover_photo_key, report.results[0].key);
        assert_ne!(report.doe.cover_photo_key.as_deref(), Some("doe/old"));
    }

    #[tokio::test]
    async fn a_supply_target_without_a_supply_id_is_a_failed_element() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let (photos, storage) = library_with(organization_id, 1);

        let report = PhotoLibraryService::new(repository_holding(photos.clone()))
            .place(
                place_command(
                    organization_id,
                    &doe,
                    vec![request(&photos[0], PlacementTargetKind::Supply)],
                ),
                &mut does_holding(doe.clone(), 0),
                &storage_service(storage.clone()),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.results[0].status, PlacementStatus::Failed);
        assert_eq!(storage.keys().len(), 1);
    }

    #[tokio::test]
    async fn a_doe_of_another_organization_is_not_found() {
        let doe = doe_in(organization(), Vec::new());

        let result = PhotoLibraryService::new(MockLibraryPhotoRepository::new())
            .place(
                place_command(organization(), &doe, Vec::new()),
                &mut does_holding(doe.clone(), 0),
                &storage_service(FakeStorage::default()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn more_than_fifty_placements_are_refused() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let ghost = photo(organization_id, "library-x/ghost.jpg");
        let placements = (0..51)
            .map(|_| request(&ghost, PlacementTargetKind::Cover))
            .collect();

        let outcome = PhotoLibraryService::new(MockLibraryPhotoRepository::new())
            .place(
                place_command(organization_id, &doe, placements),
                &mut MockDoeRepository::new(),
                &storage_service(FakeStorage::default()),
            )
            .await
            .unwrap();

        assert_eq!(outcome, Err(PhotoLibraryError::TooManyPlacements));
    }

    #[tokio::test]
    async fn a_failed_doe_write_removes_the_copies() {
        let organization_id = organization();
        let doe = doe_in(organization_id, Vec::new());
        let (photos, storage) = library_with(organization_id, 1);
        let mut does = MockDoeRepository::new();
        let held = doe.clone();
        does.expect_find_by_id().returning(move |_| {
            let doe = held.clone();
            Box::pin(async move { Ok(Some(doe)) })
        });
        does.expect_update()
            .returning(|_| Box::pin(async { Err(CoreError::Database("down".to_owned())) }));

        let result = PhotoLibraryService::new(repository_holding(photos.clone()))
            .place(
                place_command(
                    organization_id,
                    &doe,
                    vec![request(&photos[0], PlacementTargetKind::Cover)],
                ),
                &mut does,
                &storage_service(storage.clone()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Database(_))));
        assert_eq!(storage.keys(), vec![photos[0].file_key.clone()]);
    }

    #[tokio::test]
    async fn deleting_returns_the_row_for_its_objects_to_be_removed() {
        let photo = photo(organization(), "library/a.jpg");
        let mut repository = repository_holding(vec![photo.clone()]);
        repository
            .expect_delete()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let removed = PhotoLibraryService::new(repository)
            .delete(photo.organization_id, photo.id)
            .await
            .unwrap();

        assert_eq!(removed, photo);
    }

    fn real_jpeg(seed: u8) -> Vec<u8> {
        let image = image::RgbImage::from_fn(40, 30, |x, y| {
            image::Rgb([seed, (x * 6) as u8, (y * 8) as u8])
        });
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )
            .unwrap();
        bytes
    }

    fn stored(storage: &FakeStorage, key: &str, bytes: Vec<u8>) {
        storage
            .objects
            .lock()
            .unwrap()
            .insert(key.to_owned(), bytes);
    }

    fn photo_file(key: &str, caption: Option<&str>) -> DoeFile {
        DoeFile {
            key: key.to_owned(),
            caption: caption.map(str::to_owned),
        }
    }

    fn doe_with_every_kind_of_photo(organization_id: OrganizationId) -> Doe {
        let mut doe = doe_in(
            organization_id,
            vec![
                DoeSection {
                    id: Uuid::new_v4(),
                    title: "Photos".to_owned(),
                    enabled: true,
                    content: DoeSectionContent::Photos {
                        per_page: PhotosPerPage::Two,
                        photos: vec![
                            photo_file("doe/a.jpg", Some("Massif")),
                            photo_file("doe/cover.jpg", Some("Doublon de la couverture")),
                        ],
                    },
                },
                DoeSection {
                    id: Uuid::new_v4(),
                    title: "Plans".to_owned(),
                    enabled: true,
                    content: DoeSectionContent::Plans {
                        plans: vec![photo_file("doe/plan.pdf", None)],
                    },
                },
            ],
        );
        doe.cover_photo_key = Some("doe/cover.jpg".to_owned());
        doe
    }

    #[test]
    fn the_photos_of_a_doe_are_the_cover_and_the_photo_sections_without_duplicates() {
        let doe = doe_with_every_kind_of_photo(organization());

        let keys: Vec<String> = doe_photos(&doe).into_iter().map(|file| file.key).collect();

        assert_eq!(keys, vec!["doe/cover.jpg", "doe/a.jpg"]);
    }

    #[tokio::test]
    async fn the_backfill_imports_each_photo_with_its_legend_and_its_project() {
        let organization_id = organization();
        let doe = doe_with_every_kind_of_photo(organization_id);
        let storage = FakeStorage::default();
        stored(&storage, "doe/cover.jpg", real_jpeg(10));
        stored(&storage, "doe/a.jpg", real_jpeg(200));
        let project_id = doe.project_id;
        let mut repository = MockLibraryPhotoRepository::new();
        repository
            .expect_find_by_hash()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repository
            .expect_insert_if_absent()
            .returning(move |photo| {
                assert_eq!(photo.project_id, Some(project_id));
                let photo = photo.clone();
                Box::pin(async move { Ok(InsertOutcome::Inserted(photo)) })
            });

        let report = PhotoLibraryService::new(repository)
            .backfill_doe(&doe, &storage_service(storage.clone()))
            .await;

        assert_eq!(
            report,
            BackfillReport {
                imported: 2,
                ..BackfillReport::default()
            }
        );
        assert_eq!(storage.keys().len(), 2 + 4);
    }

    #[tokio::test]
    async fn a_photo_already_in_the_library_is_counted_and_not_stored_again() {
        let organization_id = organization();
        let doe = doe_with_every_kind_of_photo(organization_id);
        let storage = FakeStorage::default();
        stored(&storage, "doe/cover.jpg", real_jpeg(10));
        stored(&storage, "doe/a.jpg", real_jpeg(200));
        let existing = photo(organization_id, "library/x.jpg");
        let mut repository = MockLibraryPhotoRepository::new();
        repository.expect_find_by_hash().returning(move |_, _| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });

        let report = PhotoLibraryService::new(repository)
            .backfill_doe(&doe, &storage_service(storage.clone()))
            .await;

        assert_eq!(report.already_present, 2);
        assert_eq!(report.imported, 0);
        assert_eq!(storage.keys().len(), 2);
    }

    #[tokio::test]
    async fn a_missing_object_or_a_non_image_does_not_stop_the_others() {
        let organization_id = organization();
        let doe = doe_with_every_kind_of_photo(organization_id);
        let storage = FakeStorage::default();
        stored(&storage, "doe/cover.jpg", b"%PDF-1.4".to_vec());
        let report = PhotoLibraryService::new(inserting())
            .backfill_doe(&doe, &storage_service(storage.clone()))
            .await;

        assert_eq!(report.skipped, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.imported, 0);
    }

    #[tokio::test]
    async fn a_copied_photo_outlives_its_library_entry() {
        let organization_id = organization();
        let (photos, storage) = library_with(organization_id, 1);
        let photo = photos[0].clone();

        let key = PhotoLibraryService::new(repository_holding(photos))
            .copy_file(organization_id, photo.id, &storage_service(storage.clone()))
            .await
            .unwrap();

        assert_ne!(key, photo.file_key);
        storage.objects.lock().unwrap().remove(&photo.file_key);
        assert_eq!(
            storage.objects.lock().unwrap().get(&key),
            Some(&vec![9, 9, 9])
        );
    }

    #[tokio::test]
    async fn a_photo_of_another_organization_is_not_copied() {
        let (photos, storage) = library_with(organization(), 1);
        let id = photos[0].id;

        let result = PhotoLibraryService::new(repository_holding(photos))
            .copy_file(organization(), id, &storage_service(storage.clone()))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
        assert_eq!(storage.keys().len(), 1);
    }

    #[tokio::test]
    async fn a_photo_missing_from_storage_is_not_found() {
        let organization_id = organization();
        let photo = photo(organization_id, "library-x/gone.jpg");
        let id = photo.id;

        let result = PhotoLibraryService::new(repository_holding(vec![photo]))
            .copy_file(
                organization_id,
                id,
                &storage_service(FakeStorage::default()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
