export type CommandResult<T> =
  | {
      success: true;
      value: T;
    }
  | {
      success: false;
      value: string;
    };
