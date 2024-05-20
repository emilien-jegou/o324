import { startNewTask } from '~/api';
import { createProjectMetadata } from '~/store/projects-store';
import { SearchMenu } from '~/ui/common/search-menu';
import { MenuContent, toMenuCardData } from './menu-content';

type CreateTaskInputProps = {
  class?: string;
};

export const CreateTaskInput = (props: CreateTaskInputProps) => (
  <div class={props.class}>
    <p class="font-medium text-xs text-space-600 mx-4">Helper: task @project #tag1 #tag2</p>
    <SearchMenu
      class="mt-2 mx-4"
      placeholder="What are you working on ?"
      onSelect$={async (value: string) => {
        const selected = await toMenuCardData(value);

        if (!selected) return;
        await startNewTask({
          task_name: selected.task_name,
          project: selected.project ?? null,
          tags: selected.tags,
        });

        if (selected.project && selected.projectMetadata.isNew === true) {
          await createProjectMetadata(selected.project);
        }
      }}
      MenuContent={MenuContent}
    />
  </div>
);
