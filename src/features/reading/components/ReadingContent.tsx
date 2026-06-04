import { Segment } from "@/services/api";

interface Props {
  segmentsJson: string;
  showFurigana: boolean;
  onWordClick: (word: string, e: React.MouseEvent) => void;
}

export default function ReadingContent({ segmentsJson, showFurigana, onWordClick }: Props) {
  const paragraphs: Segment[][] = JSON.parse(segmentsJson);

  return (
    <div className={`space-y-4 leading-relaxed text-base ${showFurigana ? "" : "hide-furigana"}`}>
      <style>{`.hide-furigana rt { display: none; }`}</style>
      {paragraphs.map((para, pi) => (
        <p key={pi} className="text-justify">
          {para.map((seg, si) => {
            if (seg.k && seg.f) {
              return (
                <ruby
                  key={si}
                  className="cursor-pointer decoration-dotted underline-offset-2 hover:bg-yellow-100 dark:hover:bg-yellow-900/30"
                  onClick={(e) => onWordClick(seg.t, e)}
                >
                  {seg.t}<rt>{seg.f}</rt>
                </ruby>
              );
            }
            return <span key={si}>{seg.t}</span>;
          })}
        </p>
      ))}
    </div>
  );
}
