import { useState, useRef, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { getAudioData } from "@/services/api";

interface Props {
  audioFile: string;
  onTimeUpdate?: (time: number) => void;
  onDuration?: (duration: number) => void;
}

export default function AudioPlayer({ audioFile, onTimeUpdate, onDuration }: Props) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [speed, setSpeed] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(false);
    getAudioData(audioFile)
      .then((bytes) => {
        if (cancelled) return;
        const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
        const url = URL.createObjectURL(blob);
        setAudioUrl(url);
        setLoading(false);
      })
      .catch(() => {
        if (!cancelled) { setError(true); setLoading(false); }
      });
    return () => { cancelled = true; if (audioUrl) URL.revokeObjectURL(audioUrl); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [audioFile]);

  const togglePlay = useCallback(() => {
    if (!audioRef.current) return;
    if (audioRef.current.paused) {
      audioRef.current.play().then(() => setIsPlaying(true)).catch(() => {});
    } else {
      audioRef.current.pause();
      setIsPlaying(false);
    }
  }, []);

  const cycleSpeed = useCallback(() => {
    const speeds = [0.75, 1, 1.25, 1.5];
    const idx = speeds.indexOf(speed);
    const next = speeds[(idx + 1) % speeds.length];
    setSpeed(next);
    if (audioRef.current) audioRef.current.playbackRate = next;
  }, [speed]);

  const handleSeek = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const time = Number(e.target.value);
    setCurrentTime(time);
    if (audioRef.current) audioRef.current.currentTime = time;
  }, []);

  const fmt = (s: number) =>
    `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

  if (loading) {
    return (
      <div className="flex items-center justify-center rounded-lg border py-8">
        <p className="text-sm text-muted-foreground">加载音频中...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center rounded-lg border py-8">
        <p className="text-sm text-destructive">音频加载失败</p>
      </div>
    );
  }

  return (
    <div className="space-y-3 rounded-lg border p-4">
      <audio
        ref={audioRef}
        src={audioUrl!}
        preload="auto"
        onTimeUpdate={() => {
          const t = audioRef.current?.currentTime ?? 0;
          setCurrentTime(t);
          onTimeUpdate?.(t);
        }}
        onLoadedMetadata={() => {
          const d = audioRef.current?.duration ?? 0;
          setDuration(d);
          onDuration?.(d);
        }}
        onEnded={() => setIsPlaying(false)}
      />

      <div className="flex items-center gap-3">
        <Button variant="outline" size="icon" onClick={togglePlay} aria-label={isPlaying ? "暂停" : "播放"}>
          {isPlaying ? "⏸" : "▶"}
        </Button>

        <input
          type="range"
          min={0}
          max={duration || 0}
          value={currentTime}
          onChange={handleSeek}
          className="flex-1 cursor-pointer"
          aria-label="播放进度"
        />

        <span className="min-w-[90px] text-right text-sm tabular-nums text-muted-foreground">
          {fmt(currentTime)} / {fmt(duration)}
        </span>

        <Button variant="outline" size="sm" onClick={cycleSpeed} aria-label="语速">
          {speed}x
        </Button>
      </div>
    </div>
  );
}
