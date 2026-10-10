import { cn } from "@/lib/utils";
interface MarkProps { size?: number; className?: string }
/** Original text marks identify tools without redistributing their proprietary artwork. */
function TextMark({ text, size = 32, className }: MarkProps & { text: string }) {
 return <span aria-hidden className={cn("inline-flex shrink-0 items-center justify-center rounded-lg border border-border bg-muted font-semibold text-foreground", className)} style={{ width:size,height:size,fontSize:Math.max(8,size*0.34) }}>{text}</span>;
}
export function WorkBuddyMark(props: MarkProps) { return <TextMark {...props} text="WB" />; }
export function WorkBuddyAiMark(props: MarkProps) { return <TextMark {...props} text="W·I" />; }
export function CodeBuddyMark(props: MarkProps) { return <TextMark {...props} text="CB" />; }
export function CodeBuddyCnIdeMark(props: MarkProps) { return <TextMark {...props} text="IDE" />; }
export function CodeBuddyAiIdeMark(props: MarkProps) { return <TextMark {...props} text="I·I" />; }
export function VscodeExtMark(props: MarkProps) { return <TextMark {...props} text="VS" />; }
export function VscodeExtAiMark(props: MarkProps) { return <TextMark {...props} text="V·I" />; }
export function JetbrainsMark(props: MarkProps) { return <TextMark {...props} text="JB" />; }
export function AppIconMark({ size=32,className }: MarkProps) { return <img aria-hidden src={`${import.meta.env.BASE_URL}icon-transparent.png`} className={cn("shrink-0",className)} style={{width:size,height:size}} />; }

export function StatusDot({ on, className }: { on: boolean; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn("size-1.5 shrink-0 rounded-full", on ? "bg-primary" : "bg-muted-foreground/35", className)}
    />
  );
}
