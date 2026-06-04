#!/usr/bin/env python3
"""
Generate 15 N3 listening lessons with MP3 audio using edge-tts.

Primary path: edge-tts Python API (no pydub/ffmpeg needed for this path).
  - Stream audio bytes per sentence, collect raw MP3 data + timing metadata.
  - Concatenate sentence MP3 streams into one lesson file (validated working).
  - Parse SubMaker SRT to derive sentence durations.

Fallback: pydub silent WAV placeholders (if edge-tts has no internet).
"""

import json
import logging
import os
import sys
import traceback
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(message)s")
logger = logging.getLogger(__name__)

VOICE = "ja-JP-NanamiNeural"

# ──────────────────────────── Lesson Data ────────────────────────────

LESSONS = [
    {
        "id": "listening_001",
        "title": "駅のアナウンス",
        "sentences": [
            "電車はまもなく到着します。",
            "白線の内側までお下がりください。",
            "次は新宿駅です。",
            "出口は右側です。",
            "お忘れ物にご注意ください。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "電車が来る前に何をしなければなりませんか？",
                "options": [
                    "白線の内側まで下がる",
                    "出口を探す",
                    "忘れ物を確認する",
                    "走って電車に乗る",
                ],
                "correct_index": 0,
                "explanation": "「白線の内側までお下がりください」とアナウンスがあります。",
            }
        ],
    },
    {
        "id": "listening_002",
        "title": "天気予報",
        "sentences": [
            "今日の東京の天気は晴れです。",
            "最高気温は28度で、最低気温は20度です。",
            "午後から曇りになるでしょう。",
            "夜には雨が降る可能性があります。",
            "傘をお持ちください。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "今日の東京の最高気温は何度ですか？",
                "options": ["20度", "25度", "28度", "30度"],
                "correct_index": 2,
                "explanation": "「最高気温は28度」と言っています。",
            },
            {
                "id": "q2",
                "question": "夜の天気はどうなりますか？",
                "options": ["晴れ", "曇り", "雨", "雪"],
                "correct_index": 2,
                "explanation": "「夜には雨が降る可能性があります」と言っています。",
            },
        ],
    },
    {
        "id": "listening_003",
        "title": "買い物の会話",
        "sentences": [
            "いらっしゃいませ。",
            "すみません、この靴は何色がありますか？",
            "黒と白と茶色がございます。",
            "黒を試着してもいいですか？",
            "かしこまりました。サイズはどのくらいですか？",
            "24センチです。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "お客さんは何を買いたいですか？",
                "options": ["靴", "鞄", "帽子", "服"],
                "correct_index": 0,
                "explanation": "「この靴は何色がありますか」と聞いています。",
            }
        ],
    },
    {
        "id": "listening_004",
        "title": "道案内",
        "sentences": [
            "すみません、駅はどこですか？",
            "この道をまっすぐ行って、二つ目の信号を左に曲がってください。",
            "そうすると、右手に駅が見えます。",
            "どのくらい時間がかかりますか？",
            "歩いて10分くらいです。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "駅に行くには、どこで曲がりますか？",
                "options": ["一つ目の信号", "二つ目の信号", "三つ目の信号", "交差点"],
                "correct_index": 1,
                "explanation": "「二つ目の信号を左に曲がってください」と教えています。",
            }
        ],
    },
    {
        "id": "listening_005",
        "title": "レストランで",
        "sentences": [
            "ご注文はお決まりですか？",
            "はい、ラーメンを一つお願いします。",
            "かしこまりました。お飲み物はいかがですか？",
            "水でいいです。",
            "少々お待ちくださいませ。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "お客さんは何を注文しましたか？",
                "options": ["うどん", "ラーメン", "そば", "カレー"],
                "correct_index": 1,
                "explanation": "「ラーメンを一つお願いします」と注文しています。",
            }
        ],
    },
    {
        "id": "listening_006",
        "title": "電話の会話",
        "sentences": [
            "もしもし、田中さんのお宅ですか？",
            "はい、そうです。",
            "田中さんはいらっしゃいますか？",
            "すみません、今出かけています。",
            "何時ごろお戻りですか？",
            "6時ごろ戻ります。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "田中さんは今どこにいますか？",
                "options": ["家にいる", "出かけている", "会社にいる", "寝ている"],
                "correct_index": 1,
                "explanation": "「今出かけています」と言っています。",
            }
        ],
    },
    {
        "id": "listening_007",
        "title": "学校のお知らせ",
        "sentences": [
            "明日の授業についてお知らせします。",
            "1時間目の国語は休講になります。",
            "2時間目の数学は予定通り行います。",
            "3時間目の英語は教室が変わります。",
            "A棟の3階、302教室に集まってください。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "明日の1時間目はどうなりますか？",
                "options": ["休講", "予定通り", "教室変更", "時間変更"],
                "correct_index": 0,
                "explanation": "「1時間目の国語は休講になります」とお知らせがあります。",
            }
        ],
    },
    {
        "id": "listening_008",
        "title": "職場の会話",
        "sentences": [
            "お疲れ様です。",
            "お疲れ様です。今日の会議は何時からですか？",
            "3時からです。",
            "資料はもう準備しましたか？",
            "はい、もうコピーして配ってあります。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "会議は何時からですか？",
                "options": ["2時", "3時", "4時", "5時"],
                "correct_index": 1,
                "explanation": "「3時からです」と答えています。",
            }
        ],
    },
    {
        "id": "listening_009",
        "title": "ニュース",
        "sentences": [
            "本日、東京で新しい博物館がオープンしました。",
            "この博物館には、日本の歴史に関する展示が200点以上あります。",
            "開館時間は午前9時から午後5時までです。",
            "入場料は大人が1000円で、学生は半額です。",
            "今週末は特別に無料で入場できます。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "入場料はいくらですか？",
                "options": ["大人500円", "大人1000円", "大人1500円", "無料"],
                "correct_index": 1,
                "explanation": "「入場料は大人が1000円」と言っています。",
            },
            {
                "id": "q2",
                "question": "いつ無料で入れますか？",
                "options": ["今日", "来週", "今週末", "毎日"],
                "correct_index": 2,
                "explanation": "「今週末は特別に無料で入場できます」と言っています。",
            },
        ],
    },
    {
        "id": "listening_010",
        "title": "観光案内",
        "sentences": [
            "こちらが市内の観光マップです。",
            "この美術館はとても人気があります。",
            "入り口は向かい側にございます。",
            "見学時間は約1時間です。",
            "写真撮影は禁止されていますので、ご注意ください。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "美術館で何が禁止されていますか？",
                "options": ["食べること", "写真撮影", "電話", "大きな声"],
                "correct_index": 1,
                "explanation": "「写真撮影は禁止されています」と言っています。",
            }
        ],
    },
    {
        "id": "listening_011",
        "title": "病院で",
        "sentences": [
            "お待たせしました。どうぞお入りください。",
            "今日はどうされましたか？",
            "昨日から頭が痛くて、熱もあります。",
            "そうですか。では、まず熱を測りましょう。",
            "体温計はそこにあります。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "患者さんの症状は何ですか？",
                "options": ["お腹が痛い", "頭が痛くて熱がある", "喉が痛い", "歯が痛い"],
                "correct_index": 1,
                "explanation": "「頭が痛くて、熱もあります」と話しています。",
            }
        ],
    },
    {
        "id": "listening_012",
        "title": "図書館の案内",
        "sentences": [
            "図書館の使い方を説明します。",
            "本を借りるには、このカウンターで貸出カードを作ってください。",
            "一度に借りられるのは5冊までで、期間は2週間です。",
            "本を返すときは、この返却ポストに入れてください。",
            "開館時間は午前9時から午後8時までです。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "一度に何冊まで借りられますか？",
                "options": ["3冊", "5冊", "10冊", "無制限"],
                "correct_index": 1,
                "explanation": "「一度に借りられるのは5冊まで」と言っています。",
            }
        ],
    },
    {
        "id": "listening_013",
        "title": "バスのアナウンス",
        "sentences": [
            "このバスは渋谷駅前行きです。",
            "次の停留所は公園前です。",
            "お降りの方は、ボタンを押してお知らせください。",
            "車内で電話のご使用はご遠慮ください。",
            "まもなく公園前に到着します。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "このバスはどこ行きですか？",
                "options": ["新宿駅前", "渋谷駅前", "東京駅前", "池袋駅前"],
                "correct_index": 1,
                "explanation": "「このバスは渋谷駅前行きです」とアナウンスがあります。",
            }
        ],
    },
    {
        "id": "listening_014",
        "title": "イベントの案内",
        "sentences": [
            "来週の日曜日に日本語スピーチコンテストが行われます。",
            "参加したい方は、今週の金曜日までに申し込んでください。",
            "優勝者には図書カードが贈られます。",
            "去年は20人の参加者がいました。",
            "皆さんのご参加をお待ちしています。",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "申し込みの締め切りはいつですか？",
                "options": ["来週の日曜日", "今週の金曜日", "明日", "来月"],
                "correct_index": 1,
                "explanation": "「今週の金曜日までに申し込んでください」と言っています。",
            }
        ],
    },
    {
        "id": "listening_015",
        "title": "友達との会話",
        "sentences": [
            "今週末、一緒に映画を見に行かない？",
            "いいね。何の映画を見たい？",
            "アクション映画がいいな。",
            "私は恋愛映画の方が好きだけどな。",
            "じゃあ、両方見ようか？",
            "そうしよう！何時がいい？",
        ],
        "questions": [
            {
                "id": "q1",
                "question": "二人は最終的にどうすることにしましたか？",
                "options": [
                    "アクション映画だけ見る",
                    "恋愛映画だけ見る",
                    "両方見る",
                    "映画に行かない",
                ],
                "correct_index": 2,
                "explanation": "「両方見ようか」「そうしよう」と言っています。",
            }
        ],
    },
]

# ────────────────  Primary: edge-tts Python API  ────────────────


def generate_with_edge_tts():
    """Generate all lesson audio using the edge-tts Python API.

    For each sentence:
      - Communicate.stream_sync() yields audio (raw MP3 bytes) and metadata.
      - SubMaker collects WordBoundary / SentenceBoundary events.
      - The SRT output gives us per-sentence duration.
    All sentence audio byte streams are concatenated into one lesson MP3.
    """
    from edge_tts import Communicate, SubMaker

    audio_dir = Path("content/audio")
    audio_dir.mkdir(parents=True, exist_ok=True)

    output_lessons = []

    for lesson in LESSONS:
        lid = lesson["id"]
        title = lesson["title"]
        sentences = lesson["sentences"]
        questions = lesson["questions"]

        logger.info(f"Generating: {lid} - {title} ({len(sentences)} sentences)")

        all_audio = bytearray()
        transcript = []
        duration_total = 0.0

        for i, sentence in enumerate(sentences):
            comm = Communicate(sentence, VOICE)
            submaker = SubMaker()
            sentence_audio = bytearray()

            for chunk in comm.stream_sync():
                if chunk["type"] == "audio" and "data" in chunk:
                    sentence_audio.extend(chunk["data"])
                elif chunk["type"] in ("WordBoundary", "SentenceBoundary"):
                    submaker.feed(chunk)

            # Parse duration from SRT metadata
            dur = _srt_duration(submaker.get_srt())
            if dur <= 0:
                dur = max(len(sentence) * 0.12, 1.0)

            transcript.append(
                {
                    "text": sentence,
                    "start": round(duration_total, 3),
                    "end": round(duration_total + dur, 3),
                }
            )

            logger.info(f"  [{i+1}/{len(sentences)}] {dur:.2f}s  {sentence[:40]}")
            duration_total += dur
            all_audio.extend(sentence_audio)

        # Write merged lesson audio
        lesson_path = audio_dir / f"{lid}.mp3"
        with open(lesson_path, "wb") as f:
            f.write(bytes(all_audio))

        fsize = os.path.getsize(lesson_path)
        logger.info(f"  -> {fsize/1024:.1f} KB  |  total {duration_total:.1f}s")

        output_lessons.append(
            {
                "id": lid,
                "title": title,
                "level": "N3",
                "source": "bundled",
                "audio_file": f"audio/{lid}.mp3",
                "transcript": transcript,
                "questions": questions,
            }
        )

    _write_json(output_lessons)
    logger.info(f"\nDone! {len(output_lessons)} lessons via edge-tts.")


def _srt_duration(srt: str) -> float:
    """Extract total duration (seconds) from an edge-tts SRT string."""
    for line in srt.strip().split("\n"):
        if "-->" in line:
            parts = line.split(" --> ")
            if len(parts) == 2:
                return _ts_seconds(parts[1].strip())
    return 0.0


def _ts_seconds(ts: str) -> float:
    """Convert HH:MM:SS,mmm → float seconds."""
    try:
        h, m, rest = ts.split(":")
        s, ms = rest.split(",")
        return int(h) * 3600 + int(m) * 60 + int(s) + int(ms) / 1000
    except (ValueError, IndexError):
        return 0.0


# ────────────────  Fallback: silent WAV placeholders  ────────────────


def generate_fallback():
    """Generate silent placeholder files using pydub WAV export (no ffmpeg needed)."""
    from pydub import AudioSegment

    audio_dir = Path("content/audio")
    audio_dir.mkdir(parents=True, exist_ok=True)

    output_lessons = []

    for lesson in LESSONS:
        lid = lesson["id"]
        title = lesson["title"]
        sentences = lesson["sentences"]
        questions = lesson["questions"]

        logger.info(f"Placeholder: {lid} - {title}")

        # Build silent segments with estimated durations
        segs = []
        for sent in sentences:
            dur_s = max(len(sent) * 0.15, 1.5)
            segs.append((AudioSegment.silent(duration=int(dur_s * 1000)), sent, dur_s))

        # Merge
        merged = segs[0][0]
        for s, _, _ in segs[1:]:
            merged = merged + s

        # Build transcript
        offset = 0.0
        transcript = []
        for _, sent, dur_s in segs:
            transcript.append(
                {
                    "text": sent,
                    "start": round(offset, 3),
                    "end": round(offset + dur_s, 3),
                }
            )
            offset += dur_s

        # Export as WAV (pydub can do WAV without ffmpeg)
        lesson_path = audio_dir / f"{lid}.mp3"
        merged.export(str(lesson_path), format="wav")

        fsize = os.path.getsize(lesson_path)
        logger.info(f"  -> {fsize/1024:.1f} KB  |  {offset:.1f}s (WAV placeholder)")

        output_lessons.append(
            {
                "id": lid,
                "title": title,
                "level": "N3",
                "source": "bundled",
                "audio_file": f"audio/{lid}.mp3",
                "transcript": transcript,
                "questions": questions,
            }
        )

    _write_json(output_lessons)
    logger.info(
        f"\nDone! {len(output_lessons)} placeholders. "
        "Using silent placeholders -- real audio will need edge-tts internet access."
    )


# ────────────────  JSON output  ────────────────


def _write_json(lessons: list):
    path = Path("content/listening.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(lessons, f, ensure_ascii=False, indent=2)
    logger.info(f"Wrote {path}  ({os.path.getsize(path)} bytes)")


# ────────────────  Main  ────────────────


def main():
    # Change to project root (parent of scripts/)
    os.chdir(Path(__file__).resolve().parent.parent)
    logger.info(f"CWD: {os.getcwd()}")

    Path("content/audio").mkdir(parents=True, exist_ok=True)

    try:
        logger.info("--- edge-tts generation ---")
        generate_with_edge_tts()
    except Exception as exc:
        logger.error(f"edge-tts failed: {exc}")
        traceback.print_exc()
        logger.info("")
        logger.info("--- Fallback: silent placeholders ---")
        generate_fallback()


if __name__ == "__main__":
    main()
