use crate::{
    core::{color::SpacedRandomGenerator, storage::Storage},
    entities::project_color::ProjectColor,
};
use once_cell::sync::OnceCell;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wrap_builder::wrap_builder;

/// A thread-safe wrapper around SpacedRandomGenerator that handles
/// shared, mutable state using the interior mutability pattern (Arc<Mutex<T>>).
struct SharedColorGenerator {
    generator: Arc<Mutex<SpacedRandomGenerator>>,
}

impl SharedColorGenerator {
    /// Creates a new shared generator, pre-seeding it with existing colors.
    pub fn new(existing_colors: &[u32]) -> Self {
        let mut generator = SpacedRandomGenerator::new(360, 4.0); // max_val = 360 for hue
        for &color_hue in existing_colors {
            generator.push(color_hue);
        }

        Self {
            generator: Arc::new(Mutex::new(generator)),
        }
    }

    /// Gets the next available color hue in a thread-safe manner.
    pub fn next_number(&self) -> u32 {
        self.generator.lock().unwrap().next_number()
    }
}

#[wrap_builder(Arc)]
pub struct ProjectColorRepository {
    pub storage: Storage,
    #[builder(default)]
    color_generator: OnceCell<SharedColorGenerator>,
}

impl ProjectColorRepositoryInner {
    pub fn find(&self, projects: &[&str]) -> eyre::Result<HashMap<String, u32>> {
        self.storage.write(|qr| {
            let mut colors_bag = HashMap::<String, u32>::new();

            for &project_name in projects.iter() {
                if let Some(existing) = qr.get().primary::<ProjectColor>(project_name)? {
                    colors_bag.insert(existing.project, existing.color_hue);
                } else {
                    let generator = self.color_generator.get_or_try_init(
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

                    // Generate, save, and return the new color.
                    let new_hue = generator.next_number();

                    qr.insert(ProjectColor {
                        project: project_name.to_string(),
                        color_hue: new_hue,
                    })?;
                    colors_bag.insert(project_name.to_string(), new_hue);
                }
            }
            Ok(colors_bag)
        })
    }
}
