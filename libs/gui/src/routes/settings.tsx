import { component$, type JSXChildren } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { Input } from '~/ui/common/input';
import { InputField } from '~/ui/common/input-field';
import { Label } from '~/ui/common/label';
import { SelectField } from '~/ui/common/select-field';
import type { Classes } from '~/utils/types';

type SectionProps = {
  classes?: Classes<'root' | 'content'>;
  section: string;
  description: string;
  children: JSXChildren;
};

const Section = (props: SectionProps) => (
  <div class={twMerge('w-full flex py-8 gap-16', props.classes?.root)}>
    <div class="w-[600px]">
      <h2 class="text-md">{props.section}</h2>
      <p class="text-sm text-space-400 mt-1">{props.description}</p>
    </div>
    <div class={twMerge('w-full', props.classes?.content)}>{props.children}</div>
  </div>
);

export const Settings = component$(() => {
  //getCurrentConfig;
  return (
    <div class="relative px-12 mr-12 my-12 overflow-x-hidden">
      <h1 class="text-xl font-semibold">Settings</h1>
      <p class="text-space-400 mt-1 mb-12">Settings and options for your application</p>
      <hr class="border-space-800" />
      <Section
        section="Computer name"
        description="This can be used to identify different devices actions"
      >
        <Input name="computer-name" />
      </Section>
      <hr class="border-space-800" />
      <Section
        classes={{ content: 'flex flex-col gap-2' }}
        section="Storage"
        description="Configure how your data is stored"
      >
        <div>
          <Label text="Storage type" />
          <h3 class="pt-1 pb-6">
            <span>Git</span>
            <span class="text-space-500 pl-1"> (only supported storage)</span>
          </h3>
        </div>
        <InputField
          required
          label="Remote origin url"
          name="remote-origin"
          info="Path of the remote directory where tasks should be persisted"
        />
        <InputField
          required
          label="Git storage path"
          name="storage-path"
          info="Path of git directory where tasks are stored (default to ~/.local/share/o324/git-storage-data)"
        />
        <InputField
          required
          label="Connection name"
          name="connection-name"
          info="Name of the connection, will appear in commit history"
        />
        <SelectField
          required
          label="Data file format"
          name="connection-name"
          options={[
            { value: 'json', label: 'json' },
            { value: 'yaml', label: 'yaml' },
            { value: 'toml', label: 'toml' },
          ]}
          info="Format of document in the storage directory"
        />
      </Section>
      <div class="fixed left-0 bottom-0 w-full bg-space-800 py-4">
        <div class="flex w-fit ml-auto">
          <p>Apply 3 changes</p>
        </div>
      </div>
    </div>
  );
});

//    /// Storage format of the files (default to: json)
//    pub git_file_format_type: Option<String>,
