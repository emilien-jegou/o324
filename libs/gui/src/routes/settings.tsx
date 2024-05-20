import { twMerge } from 'tailwind-merge';
import { Input } from '~/ui/common/input';
import { InputField } from '~/ui/common/input-field';
import { Label } from '~/ui/common/label';
import { SelectField } from '~/ui/common/select-field';
import type { JSXChildren } from '@builder.io/qwik';
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

export const Settings = () => {
  return (
    <div class="px-12 mr-12 my-8 mt-16">
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
        classes={{ content: 'flex flex-col gap-8' }}
        section="Storage"
        description="Configure how your data is stored"
      >
        <div>
          <Label text="Storage type" />
          <h3 class="pt-1 pb-2">
            <span>Git</span>
            <span class="text-space-400 pl-1"> (only supported storage)</span>
          </h3>
        </div>
        <InputField
          required
          label="Remote origin url"
          name="remote-origin"
          helperText="Path of the remote directory where tasks should be persisted"
        />
        <InputField
          required
          label="Git storage path"
          name="storage-path"
          helperText="Path of git directory where tasks are stored (default to ~/.local/share/o324/git-storage-data)"
        />
        <InputField
          required
          label="Connection name"
          name="connection-name"
          helperText="Name of the connection, will appear in commit history"
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
          helperText="Format of document in the storage directory"
        />
      </Section>
    </div>
  );
};

//    /// Storage format of the files (default to: json)
//    pub git_file_format_type: Option<String>,
