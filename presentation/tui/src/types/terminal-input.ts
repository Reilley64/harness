type Timing = {
  startedAt: number;
  receivedAt: number;
};

export type TerminalMouse = Timing & {
  type: "mouse";
  button: number;
  x: number;
  y: number;
  released: boolean;
  wheel: number;
};

export type TerminalEvent =
  | TerminalMouse
  | (Timing & {
      type: "shortcut";
      action: "toggle" | "griller" | "children";
    })
  | (Timing & { type: "focus"; focused: boolean })
  | (Timing & { type: "reply" });
