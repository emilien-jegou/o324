use crate::{
    core::{
        batch_loader::{BatchCall, BatchLoader},
        color::SpacedRandomGenerator,
        storage::Storage,
    },
    entities::project_color::ProjectColor,
};
use derive_more::Deref;
use futures::{future::BoxFuture, FutureExt};
use once_cell::sync::OnceCell;
use std::{collections::HashMap, sync::Arc};
use typed_builder::TypedBuilder;

/// A thread-safe wrapper around SpacedRandomGenerator that handles
/// shared, mutable state using the interior mutability pattern (Arc<Mutex<T>>).
#[derive(Deref, Clone)]
#[deref(forward)]
struct SharedColorGenerator(Arc<SpacedRandomGenerator>);

impl SharedColorGenerator {
    pub fn new(existing_colors: &[u32]) -> Self {
        let generator = SpacedRandomGenerator::new(360, 4.0); // max_val = 360 for hue
        for &color_hue in existing_colors {
            generator.push(color_hue);
        }

        Self(Arc::new(generator))
    }

    pub fn next_number(&self) -> u32 {
        self.0.next_number()
    }
}

struct ProjectColorBatcher {
    pub storage: Storage,
    pub color_generator: OnceCell<SharedColorGenerator>,
}

impl BatchCall<String, u32> for ProjectColorBatcher {
    fn call(
        &self,
        projects: Vec<String>,
    ) -> BoxFuture<'static, eyre::Result<HashMap<String, u32>>> {
        let storage = self.storage.clone();
        let color_generator = self.color_generator.clone();

        async move {
            storage.write_txn(move |qr| {
                let mut results = HashMap::new();
                for project in projects.into_iter() {
                    if let Some(existing) = qr.get().primary::<ProjectColor>(project.clone())? {
                        results.insert(existing.project, existing.color_hue);
                    } else {
                        let generator = color_generator.get_or_try_init(
                            || -> eyre::Result<SharedColorGenerator> {
                                let all_colors = qr
                                    .scan()
                                    .primary::<ProjectColor>()?
                                    .all()?
                                    .map(|res| res.map(|pc| pc.color_hue))
                                    .collect::<Result<Vec<u32>, _>>()?;

                                Ok(SharedColorGenerator::new(&all_colors))
                            },
                        )?;

                        let color_hue = generator.next_number();

                        qr.insert(ProjectColor {
                            project: project.to_string(),
                            color_hue,
                        })?;

                        results.insert(project, color_hue);
                    }
                }
                Ok(results)
            })
        }
        .boxed()
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = ProjectColorRepository))]
pub struct ProjectColorRepositoryBuilder {
    pub storage: Storage,
}

pub struct ProjectColorRepository {
    pub color_loader: BatchLoader<String, u32>,
}

impl From<ProjectColorRepositoryBuilder> for ProjectColorRepository {
    fn from(val: ProjectColorRepositoryBuilder) -> Self {
        let color_loader = BatchLoader::new(ProjectColorBatcher {
            storage: val.storage,
            color_generator: Default::default(),
        });

        ProjectColorRepository { color_loader }
    }
}

impl ProjectColorRepository {
    pub fn builder() -> ProjectColorRepositoryBuilderBuilder {
        ProjectColorRepositoryBuilder::builder()
    }

    pub async fn get(&self, project: &str) -> eyre::Result<u32> {
        let result = self.color_loader.run(&[project.to_string()]).await?;
        Ok(*result.get(project).expect("internal logic error"))
    }

    pub async fn get_many(&self, projects: &[&str]) -> eyre::Result<HashMap<String, u32>> {
        let project_keys: Vec<String> = projects.iter().map(|s| s.to_string()).collect();
        self.color_loader.run(&project_keys).await
    }
}
