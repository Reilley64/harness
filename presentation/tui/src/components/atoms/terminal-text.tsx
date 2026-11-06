import { Text } from "ink";
import type { ComponentProps } from "react";
import { foregroundColors } from "../../theme.js";

type Props = Omit<ComponentProps<typeof Text>, "children"> & {
  children: string;
};

export function TerminalText({ children, ...props }: Props) {
  return (
    <Text
      {...props}
      color={
        props.color ? (foregroundColors[props.color] ?? props.color) : undefined
      }
    >
      {children.replace(/[\u0000-\u001f\u007f-\u009f]/g, "")}
    </Text>
  );
}
