import { $, useSignal, component$, useTask$, useResource$, Resource } from '@builder.io/qwik';
import * as R from 'remeda';
import { z } from 'zod';
import { editTask, type Task } from '~/api';
import { useForm, validate } from '~/hooks/use-form';
import { useTaskContext } from '~/provider/task-context';
import { AutocompleteInput } from '~/ui/common/autocomplete-input';
import { Button } from '~/ui/common/button';
import { Field } from '~/ui/common/field';
import { InputField } from '~/ui/common/input-field';
import { Modal } from '~/ui/common/modal';
import { MultiSelect } from '~/ui/common/multi-select';
import { GrowingMultiSelect } from './custom-multi-select';
import type { Signal } from '@builder.io/qwik';
import type { MultiSelectOption } from '~/ui/common/multi-select';

type EditTaskModalProps = {
  'bind:show': Signal<boolean>;
  task: Task;
};

const editTaskSchema = z.object({
  task_name: z.string().min(1, 'Task must have a name').optional(),
  project: z.string().optional(),
  tags: z.array(z.string()).optional(),
});

type ReachOutFormLogicData = z.infer<typeof editTaskSchema>;

export const EditTaskModal = component$((props: EditTaskModalProps) => {
  const taskContext = useTaskContext();
  const tagsValue = useSignal(props.task.tags);
  const project = useSignal<string>(props.task.project ?? '');

  const data = useResource$(() => {
    const projects = new Set<string>();
    const tags = new Set<string>();

    Object.values(taskContext.tasks).forEach((task) => {
      task.project && projects.add(task.project);
      task.tags.forEach((tag) => tags.add(tag));
    });

    return {
      projectOptions: [...projects],
      tagsOptions: [...tags].map((t) => ({ value: t, label: t })),
    };
  });

  const form = useForm<ReachOutFormLogicData>('edit-task-form', {
    defaultValue: {
      task_name: props.task.task_name,
      project: props.task.project ?? undefined,
      tags: props.task.tags,
    },
    onSuccess$: $(async (data: ReachOutFormLogicData) => {
      console.info(data);
      await editTask(props.task.ulid, {
        task_name: data.task_name ?? null,
        project: data.project ?? null,
        tags: data.tags ?? null,
      });
      props['bind:show'].value = false;
    }),
    validate: validate($(editTaskSchema)),
  });

  return (
    <Modal bind:show={props['bind:show']} contentClass="w-[800px] h-auto">
      <h2 class="text-lg font-medium mb-6 text-ellipsis overflow-hidden">Edit task</h2>
      <Resource
        value={data}
        onResolved={({ projectOptions, tagsOptions }) => (
          <form class="flex gap-1 flex-col" preventdefault:submit onSubmit$={form.formSubmit$}>
            <InputField
              name="name"
              value={form.fields.task_name}
              error={form.errors.task_name}
              autoFocus
              onInput$={$((value: string) => {
                form.setFormValue$('task_name', value);
              })}
              label="Project name"
              autocomplete="email"
            />
            <Field error={form.errors.project} label="Project">
              <AutocompleteInput
                name="project"
                bind:value={project}
                onInput$={(value) => {
                  form.setFormValue$('project', value);
                }}
                options={projectOptions}
              />
            </Field>
            <Field error={form.errors.tags} label="Task tags">
              <GrowingMultiSelect
                name="tags"
                error={!!form.errors.tags}
                bind:value={tagsValue}
                class="w-full"
                onChange$={(d) => form.setFormValue$('tags', d)}
                placeholder="search tags..."
                defaultOptions={tagsOptions}
              />
            </Field>
            <div class="flex flex-col-reverse mt-2 sm:flex-row sm:justify-end sm:space-x-2">
              <Button
                variant="outlined"
                onClick$={() => {
                  props['bind:show'].value = false;
                }}
                class="mt-2 sm:mt-0"
              >
                Cancel
              </Button>
              <Button type="submit" class="hover:bg-accent-700">
                Update
              </Button>
            </div>
          </form>
        )}
      />
    </Modal>
  );
});
