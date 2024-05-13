export type Task = {
  ulid: string,
  end: number | null,
  project: string | null,
  start: number,
  tags: string[],
  task_name: string,
  __hash: number
}

