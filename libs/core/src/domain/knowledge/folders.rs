//! The folder tree of the drive: names unique among siblings, at most
//! `MAX_FOLDER_DEPTH` levels, never a folder inside itself.

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CreateKnowledgeFolderCommand, KnowledgeArticleId, KnowledgeError, KnowledgeFolder,
    KnowledgeFolderId, KnowledgeFolderView, KnowledgeOutcome, OrganizationId,
    UpdateKnowledgeFolderCommand,
    domain::knowledge::{
        ports::{KnowledgeArticleRepository, KnowledgeFolderRepository},
        text::{MAX_FOLDER_DEPTH, MAX_MOVED_ARTICLES, normalize_folder_name},
    },
};

pub struct KnowledgeFolderService<F>
where
    F: KnowledgeFolderRepository,
{
    folders: F,
}

impl<F> KnowledgeFolderService<F>
where
    F: KnowledgeFolderRepository,
{
    pub fn new(folders: F) -> Self {
        Self { folders }
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<KnowledgeFolder, CoreError> {
        self.folders
            .find_by_id(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    /// Not found when the folder is missing or held by another organization.
    pub async fn require(
        &mut self,
        organization_id: OrganizationId,
        id: Option<KnowledgeFolderId>,
    ) -> Result<(), CoreError> {
        match id {
            Some(id) => self.get(organization_id, id).await.map(|_| ()),
            None => Ok(()),
        }
    }

    pub async fn browse(
        &mut self,
        organization_id: OrganizationId,
        id: Option<KnowledgeFolderId>,
    ) -> Result<KnowledgeFolderView, CoreError> {
        let (folder, path) = match id {
            Some(id) => {
                let folder = self.get(organization_id, id).await?;
                let path = self.folders.path_to(organization_id, id).await?;
                (Some(folder), path)
            }
            None => (None, Vec::new()),
        };
        let children = self.folders.children(organization_id, id).await?;
        Ok(KnowledgeFolderView {
            folder,
            path,
            children,
        })
    }

    pub async fn create(
        &mut self,
        command: CreateKnowledgeFolderCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeFolder>, CoreError> {
        let name = match normalize_folder_name(&command.name) {
            Ok(name) => name,
            Err(error) => return Ok(Err(error)),
        };
        let organization_id = command.organization_id;
        if let Some(parent_id) = command.parent_id {
            self.get(organization_id, parent_id).await?;
            let depth = self
                .folders
                .path_to(organization_id, parent_id)
                .await?
                .len()
                + 1;
            if depth > MAX_FOLDER_DEPTH {
                return Ok(Err(KnowledgeError::FolderTooDeep));
            }
        }
        if self
            .name_taken(organization_id, command.parent_id, &name, None)
            .await?
        {
            return Ok(Err(KnowledgeError::FolderNameTaken));
        }
        let now = Utc::now();
        let folder = KnowledgeFolder {
            id: KnowledgeFolderId(generate_uuid_v7()),
            organization_id,
            parent_id: command.parent_id,
            name,
            created_at: now,
            updated_at: now,
        };
        self.folders.insert(&folder).await.map(Ok)
    }

    pub async fn update(
        &mut self,
        command: UpdateKnowledgeFolderCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeFolder>, CoreError> {
        let organization_id = command.organization_id;
        let mut folder = self.get(organization_id, command.id).await?;
        if let Some(name) = &command.name {
            folder.name = match normalize_folder_name(name) {
                Ok(name) => name,
                Err(error) => return Ok(Err(error)),
            };
        }
        if let Some(parent_id) = command.parent_id
            && parent_id != folder.parent_id
        {
            if let Err(error) = self.check_move(&folder, parent_id).await? {
                return Ok(Err(error));
            }
            folder.parent_id = parent_id;
        }
        if self
            .name_taken(
                organization_id,
                folder.parent_id,
                &folder.name,
                Some(folder.id),
            )
            .await?
        {
            return Ok(Err(KnowledgeError::FolderNameTaken));
        }
        folder.updated_at = Utc::now();
        self.folders.update(&folder).await.map(Ok)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<KnowledgeOutcome<()>, CoreError> {
        self.get(organization_id, id).await?;
        if !self.folders.is_empty(organization_id, id).await? {
            return Ok(Err(KnowledgeError::FolderNotEmpty));
        }
        self.folders.delete(organization_id, id).await.map(Ok)
    }

    /// Moves the articles into a folder, or to the root. Every article must
    /// belong to the organization, or none moves.
    pub async fn move_articles<A>(
        &mut self,
        articles: &mut A,
        organization_id: OrganizationId,
        ids: &[KnowledgeArticleId],
        folder_id: Option<KnowledgeFolderId>,
    ) -> Result<KnowledgeOutcome<u64>, CoreError>
    where
        A: KnowledgeArticleRepository,
    {
        if ids.len() > MAX_MOVED_ARTICLES {
            return Ok(Err(KnowledgeError::TooManyArticles));
        }
        self.require(organization_id, folder_id).await?;
        let mut unique = ids.to_vec();
        unique.sort_by_key(|id| id.0);
        unique.dedup();
        let moved = articles
            .move_to_folder(organization_id, &unique, folder_id)
            .await?;
        if moved != unique.len() as u64 {
            return Err(CoreError::NotFound);
        }
        Ok(Ok(moved))
    }

    async fn check_move(
        &mut self,
        folder: &KnowledgeFolder,
        parent_id: Option<KnowledgeFolderId>,
    ) -> Result<KnowledgeOutcome<()>, CoreError> {
        let Some(parent_id) = parent_id else {
            return Ok(Ok(()));
        };
        if parent_id == folder.id {
            return Ok(Err(KnowledgeError::FolderCycle));
        }
        self.get(folder.organization_id, parent_id).await?;
        let path = self
            .folders
            .path_to(folder.organization_id, parent_id)
            .await?;
        if path.iter().any(|ancestor| ancestor.id == folder.id) {
            return Ok(Err(KnowledgeError::FolderCycle));
        }
        let height = self
            .folders
            .height(folder.organization_id, folder.id)
            .await? as usize;
        if path.len() + 1 + height > MAX_FOLDER_DEPTH {
            return Ok(Err(KnowledgeError::FolderTooDeep));
        }
        Ok(Ok(()))
    }

    async fn name_taken(
        &mut self,
        organization_id: OrganizationId,
        parent_id: Option<KnowledgeFolderId>,
        name: &str,
        except: Option<KnowledgeFolderId>,
    ) -> Result<bool, CoreError> {
        let name = name.to_lowercase();
        Ok(self
            .folders
            .children(organization_id, parent_id)
            .await?
            .iter()
            .any(|sibling| Some(sibling.id) != except && sibling.name.to_lowercase() == name))
    }
}

#[cfg(test)]
mod tests {
    use authz::Subject;

    use super::*;
    use crate::domain::knowledge::ports::{
        MockKnowledgeArticleRepository, MockKnowledgeFolderRepository,
    };

    fn org() -> OrganizationId {
        OrganizationId(generate_uuid_v7())
    }

    fn folder(
        organization_id: OrganizationId,
        parent_id: Option<KnowledgeFolderId>,
        name: &str,
    ) -> KnowledgeFolder {
        let now = Utc::now();
        KnowledgeFolder {
            id: KnowledgeFolderId(generate_uuid_v7()),
            organization_id,
            parent_id,
            name: name.to_owned(),
            created_at: now,
            updated_at: now,
        }
    }

    /// A chain of `depth` folders, each inside the previous one.
    fn chain(organization_id: OrganizationId, depth: usize) -> Vec<KnowledgeFolder> {
        let mut path: Vec<KnowledgeFolder> = Vec::new();
        for level in 0..depth {
            let parent = path.last().map(|f| f.id);
            path.push(folder(organization_id, parent, &format!("n{level}")));
        }
        path
    }

    fn knowing(folders: Vec<KnowledgeFolder>) -> MockKnowledgeFolderRepository {
        let mut repository = MockKnowledgeFolderRepository::new();
        let known = folders.clone();
        repository.expect_find_by_id().returning(move |_, id| {
            let found = known.iter().find(|f| f.id == id).cloned();
            Box::pin(async move { Ok(found) })
        });
        let known = folders.clone();
        repository.expect_path_to().returning(move |_, id| {
            let mut path = Vec::new();
            let mut current = known.iter().find(|f| f.id == id).cloned();
            while let Some(folder) = current {
                current = folder
                    .parent_id
                    .and_then(|parent| known.iter().find(|f| f.id == parent).cloned());
                path.insert(0, folder);
            }
            Box::pin(async move { Ok(path) })
        });
        let known = folders;
        repository.expect_children().returning(move |_, parent| {
            let children: Vec<_> = known
                .iter()
                .filter(|f| f.parent_id == parent)
                .cloned()
                .collect();
            Box::pin(async move { Ok(children) })
        });
        repository
    }

    fn actor() -> Subject {
        Subject::new(authz::SubjectKind::User, "someone")
    }

    #[tokio::test]
    async fn a_sibling_with_the_same_name_in_another_case_is_refused() {
        let organization_id = org();
        let existing = folder(organization_id, None, "Devis");
        let outcome = KnowledgeFolderService::new(knowing(vec![existing]))
            .create(CreateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                parent_id: None,
                name: " devis ".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderNameTaken);
    }

    #[tokio::test]
    async fn the_eleventh_level_is_refused() {
        let organization_id = org();
        let path = chain(organization_id, MAX_FOLDER_DEPTH);
        let deepest = path.last().unwrap().id;
        let outcome = KnowledgeFolderService::new(knowing(path))
            .create(CreateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                parent_id: Some(deepest),
                name: "trop".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderTooDeep);
    }

    #[tokio::test]
    async fn the_tenth_level_is_accepted() {
        let organization_id = org();
        let path = chain(organization_id, MAX_FOLDER_DEPTH - 1);
        let deepest = path.last().unwrap().id;
        let mut repository = knowing(path);
        repository.expect_insert().returning(|folder| {
            let folder = folder.clone();
            Box::pin(async move { Ok(folder) })
        });
        let created = KnowledgeFolderService::new(repository)
            .create(CreateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                parent_id: Some(deepest),
                name: "dernier".to_owned(),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(created.parent_id, Some(deepest));
    }

    #[tokio::test]
    async fn a_folder_cannot_move_under_its_own_subfolder() {
        let organization_id = org();
        let path = chain(organization_id, 3);
        let (top, bottom) = (path[0].id, path[2].id);
        let outcome = KnowledgeFolderService::new(knowing(path))
            .update(UpdateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                id: top,
                name: None,
                parent_id: Some(Some(bottom)),
            })
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderCycle);
    }

    #[tokio::test]
    async fn a_folder_cannot_move_into_itself() {
        let organization_id = org();
        let path = chain(organization_id, 1);
        let id = path[0].id;
        let outcome = KnowledgeFolderService::new(knowing(path))
            .update(UpdateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                id,
                name: None,
                parent_id: Some(Some(id)),
            })
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderCycle);
    }

    #[tokio::test]
    async fn moving_a_tree_counts_the_levels_below_it() {
        let organization_id = org();
        let deep = chain(organization_id, 8);
        let target = deep.last().unwrap().id;
        let moved = folder(organization_id, None, "branche");
        let moved_id = moved.id;
        let mut all = deep;
        all.push(moved);
        let mut repository = knowing(all);
        repository
            .expect_height()
            .returning(|_, _| Box::pin(async { Ok(2) }));
        let outcome = KnowledgeFolderService::new(repository)
            .update(UpdateKnowledgeFolderCommand {
                actor: actor(),
                organization_id,
                id: moved_id,
                name: None,
                parent_id: Some(Some(target)),
            })
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderTooDeep);
    }

    #[tokio::test]
    async fn a_folder_that_holds_something_is_not_deleted() {
        let organization_id = org();
        let full = folder(organization_id, None, "Plein");
        let id = full.id;
        let mut repository = knowing(vec![full]);
        repository
            .expect_is_empty()
            .returning(|_, _| Box::pin(async { Ok(false) }));
        let outcome = KnowledgeFolderService::new(repository)
            .delete(organization_id, id)
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::FolderNotEmpty);
    }

    #[tokio::test]
    async fn an_article_of_another_organization_stops_the_whole_move() {
        let organization_id = org();
        let target = folder(organization_id, None, "Cible");
        let target_id = target.id;
        let mut articles = MockKnowledgeArticleRepository::new();
        articles
            .expect_move_to_folder()
            .returning(|_, _, _| Box::pin(async { Ok(1) }));
        let ids = [
            KnowledgeArticleId(generate_uuid_v7()),
            KnowledgeArticleId(generate_uuid_v7()),
        ];
        let result = KnowledgeFolderService::new(knowing(vec![target]))
            .move_articles(&mut articles, organization_id, &ids, Some(target_id))
            .await;
        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
