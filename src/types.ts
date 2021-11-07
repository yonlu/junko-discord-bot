export interface IInteraction {
  isCommand: () => boolean;
  reply: (input: { content: string; ephemeral?: boolean }) => Promise<string>;
  commandName: string;
}
