import { useMemo, useRef, useEffect } from "react";
import { TranscriptEntry } from "@/services/api";

interface Props {
  transcriptJson: string;
  currentTime: number;
  onSeek: (time: number) => void;
}

export default function TranscriptView({ transcriptJson, currentTime, onSeek }: Props) {
  const entries: TranscriptEntry[] = useMemo(() => {
    try { return JSON.parse(transcriptJson); } catch { return []; }
  }, [transcriptJson]);

  const activeIndex = entries.findIndex(
    (e) => currentTime >= e.start && currentTime < e.end,
  );
  const activeRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (activeRef.current) {
      activeRef.current.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }, [activeIndex]);

  if (entries.length === 0) return null;

  return (
    <div className="space-y-2">
      <h3 className="font-semibold">字幕</h3>
      <div className="max-h-64 space-y-1 overflow-y-auto rounded-md border p-3">
        {entries.map((entry, i) => (
          <p
            key={i}
            ref={i === activeIndex ? activeRef : undefined}
            className={`cursor-pointer rounded px-2 py-1 text-sm transition-colors ${
              i === activeIndex
                ? "bg-primary text-primary-foreground font-medium"
                : "text-muted-foreground hover:bg-muted"
            }`}
            onClick={() => onSeek(entry.start)}
          >
            {entry.text}
          </p>
        ))}
      </div>
    </div>
  );
}
