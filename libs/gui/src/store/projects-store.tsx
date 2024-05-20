import { isSome, none, some, type Option } from '~/utils/option';
import { getKeyedStore } from '~/utils/store';

export type ProjectColor = { name: string; light: string; dark: string };

export type ProjectMetadata = {
  projectColor: ProjectColor;
  isNew?: boolean;
};

const projectsStore = getKeyedStore<Option<ProjectMetadata>>('projects', none());

(window as any).__STORE_PROJECTS = projectsStore;

const genColor = (name: string, color: string): ProjectColor => ({
  name,
  light: color,
  dark: color,
});

// TODO: get color from tailwind, don't store raw values
const projectColorPool: ProjectColor[] = [
  genColor('red', '#FFADAD'),
  genColor('orange', '#FFD6A5'),
  genColor('salmon', '#fcd5ce'),
  genColor('yellow', '#FDFFB6'),
  genColor('light-green', '#CAFFBF'),
  genColor('mint', '#d3f8e2'),
  genColor('cyan', '#9BF6FF'),
  genColor('purple', '#BDB2FF'),
  genColor('pink', '#FFC6FF'),
  genColor('cactus', '#e9edc9'),
  genColor('taupe', '#efe5dc'),
];

const defaultColor = genColor('grey', '#D3D6EB');

export const getProjectMetadata = async (project_name: string) => {
  const localStore = projectsStore.at(project_name);
  const project = await localStore.get();

  if (isSome(project)) {
    return project.value;
  } else {
    return { projectColor: defaultColor, isNew: true };
  }
};

export const createProjectMetadata = async (project_name: string): Promise<ProjectMetadata> => {
  const localStore = projectsStore.at(project_name);
  const projects = await projectsStore.getAll();
  const takenColors = new Set(
    Object.values(projects)
      .filter(isSome)
      .map((x) => x.value.projectColor.name),
  );

  let availableColors = projectColorPool.filter(({ name }) => !takenColors.has(name));
  if (availableColors.length === 0) {
    availableColors = projectColorPool;
  }
  const randomColorIndex = Math.floor(Math.random() * availableColors.length);
  const chosenColor = availableColors[randomColorIndex];
  const metadata: ProjectMetadata = {
    projectColor: chosenColor,
  };

  localStore.set(some(metadata));
  return metadata;
};

export const getOrCreateProjectMetadata = async (project_name: string) => {
  const metadata = await getProjectMetadata(project_name);
  if (metadata.isNew) {
    return createProjectMetadata(project_name);
  }
  return metadata;
};
