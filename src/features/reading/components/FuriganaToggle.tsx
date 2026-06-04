import { Button } from "@/components/ui/button";

interface Props {
  show: boolean;
  onToggle: () => void;
}

export default function FuriganaToggle({ show, onToggle }: Props) {
  return (
    <Button variant="outline" size="sm" onClick={onToggle}>
      {show ? "振り仮名: 表示" : "振り仮名: 非表示"}
    </Button>
  );
}
