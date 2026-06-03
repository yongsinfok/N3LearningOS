# JLPT N3 Learning System PRD

## 1. Project Overview

### Project Name

JLPT N3 Learning System

### Goal

Develop a fully offline, local-first Japanese learning platform focused on JLPT N3 preparation.

The application should provide:

* Vocabulary learning
* Grammar learning
* Kanji learning
* Reading practice
* Listening practice
* Mock examinations
* Spaced Repetition System (SRS)

The system is intended for personal use and must not depend on cloud services.

---

# 2. Product Objectives

## Primary Objectives

* Prepare for JLPT N3 efficiently
* Centralize all study materials in one application
* Support long-term learning habits
* Provide exam-oriented training
* Work completely offline

## Non-Goals

* No AI features
* No user accounts
* No cloud sync
* No payment system
* No multiplayer functionality
* No social features

---

# 3. Technology Stack

## Backend

* Rust
* Tauri v2
* SQLx
* SQLite
* Tokio
* Serde

## Frontend

* React
* TypeScript
* Vite
* TailwindCSS
* shadcn/ui
* Zustand
* TanStack Query

## Database

* SQLite

## Packaging

* Windows Desktop Application
* Single Executable Distribution

---

# 4. Learning Content Sources

All imported data must preserve original source attribution.

---

## Vocabulary

### Source

Open Anki JLPT Deck

URL

https://github.com/jamsinclair/open-anki-jlpt-decks

### Data Structure

```ts
{
  id: string
  word: string
  reading: string
  meaning: string
  example: string
  source: string
}
```

---

## Grammar

### Source

JLPT Sensei N3 Grammar

URL

https://jlptsensei.com/jlpt-n3-grammar-list/

### Data Structure

```ts
{
  id: string
  grammar: string
  meaning: string
  explanation: string
  examples: string[]
  source: string
}
```

---

## Kanji

### Source

JLPT Benkyo

URL

https://jlptbenkyo.com/

### Data Structure

```ts
{
  id: string
  kanji: string
  onyomi: string[]
  kunyomi: string[]
  meaning: string
  source: string
}
```

---

## Reading

### Source

NHK Easy News

URL

https://www3.nhk.or.jp/news/easy/

### Data Structure

```ts
{
  id: string
  title: string
  content: string
  articleUrl: string
  source: string
}
```

### Features

* Furigana Toggle
* Vocabulary Highlight
* Grammar Highlight
* Bookmark Article

---

## Listening

### Source

Nihongo no Mori

URL

https://www.youtube.com/@nihongonomori2013

### Data Structure

```ts
{
  id: string
  title: string
  youtubeUrl: string
  transcript: string
  source: string
}
```

### Features

* Transcript View
* Repeat Segment
* Playback Speed Control
* Bookmark Lesson

---

## Mock Exams

### Source

MLC Japanese

URL

https://www.mlcjapanese.co.jp/

### Sections

* Vocabulary
* Grammar
* Reading
* Listening
* Mixed Exam

---

# 5. Core Features

## Dashboard

### Display

* Daily Study Time
* Current Study Streak
* Weekly Progress
* Monthly Progress
* Completion Percentage
* Weak Area Statistics

---

# 6. Vocabulary Module

## Features

### Flashcards

Front

```txt
勉強
```

Back

```txt
べんきょう
study
example sentence
```

### Quiz Modes

* Japanese → English
* English → Japanese
* Reading Selection
* Fill in the Blank
* Multiple Choice

### Additional Features

* Search
* Bookmark
* Tags
* Progress Tracking

---

# 7. Grammar Module

## Features

### Grammar Detail Page

* Grammar Pattern
* Meaning
* Explanation
* Examples
* Related Grammar

### Functions

* Search
* Bookmark
* Notes
* Quiz

---

# 8. Kanji Module

## Features

### Kanji Detail Page

* Character
* Meaning
* Onyomi
* Kunyomi
* Example Words

### Functions

* Search
* Bookmark
* Writing Practice Canvas
* Stroke Order SVG
* Quiz

---

# 9. Reading Module

## Features

### Reader Mode

* Furigana Toggle
* Vocabulary Lookup
* Grammar Highlighting
* Bookmark Unknown Words

### Reading Quiz

* Vocabulary Questions
* Grammar Questions
* Reading Comprehension

---

# 10. Listening Module

## Features

### Lesson Player

* Embedded Video
* Transcript Display
* Playback Speed
* Repeat Sentence
* Repeat Segment

### Listening Quiz

* Listening Comprehension
* Vocabulary Recognition
* Grammar Recognition

---

# 11. Mock Exam Module

## Sections

### Vocabulary

文字・語彙

### Grammar

文法

### Reading

読解

### Listening

聴解

### Mixed Exam

Full JLPT N3 Simulation

---

## Features

* Timer
* Auto Scoring
* Review Incorrect Answers
* Exam History
* Performance Analysis

---

# 12. Spaced Repetition System

## Algorithm

Primary:

FSRS

Fallback:

SM-2

---

## Review Buttons

* Again
* Hard
* Good
* Easy

---

## Review Tracking

Track:

* Review History
* Card Stability
* Retention Rate
* Due Dates

---

# 13. Data Import System

## Supported Formats

* CSV
* JSON
* APKG (Anki)

---

## Import Modules

```txt
src-tauri/src/importers/

anki_importer.rs
csv_importer.rs
json_importer.rs
nhk_importer.rs
grammar_importer.rs
kanji_importer.rs
```

---

# 14. Database Design

## Core Tables

### Content

* vocabulary
* grammar
* kanji
* reading
* listening

### Learning

* flashcard
* review_history
* study_session

### Tracking

* mistake_log
* bookmark
* note

### Statistics

* daily_stats
* streak_stats

---

# 15. User Interface

## Design Inspiration

* Duolingo
* Bunpro
* Anki

---

## Requirements

* Responsive Design
* Dark Mode
* Light Mode
* Keyboard Friendly
* Fast Navigation
* Accessibility Support

---

# 16. File Structure

```txt
jlpt-n3-app/

src/
├── features/
│   ├── dashboard/
│   ├── vocabulary/
│   ├── grammar/
│   ├── kanji/
│   ├── reading/
│   ├── listening/
│   ├── exams/
│   └── settings/
│
├── components/
├── hooks/
├── stores/
├── services/
├── lib/
└── routes/

src-tauri/
├── src/
│   ├── commands/
│   ├── database/
│   ├── importers/
│   ├── fsrs/
│   ├── models/
│   └── services/
│
└── migrations/
```

---

# 17. MVP Scope

Phase 1

* Database
* Vocabulary
* Flashcards
* FSRS
* Dashboard

Phase 2

* Grammar
* Kanji
* Search
* Notes

Phase 3

* Reading
* Listening
* Import Pipeline

Phase 4

* Mock Exams
* Analytics
* Statistics

Phase 5

* N2 Expansion
* N1 Expansion

---

# 18. Success Metrics

## Learning Metrics

* Daily Study Time
* Study Streak
* Vocabulary Retention
* Grammar Completion
* Kanji Completion

## Exam Metrics

* Quiz Accuracy
* Mock Exam Score
* Weakness Categories

---

# 19. Future Expansion

Support:

* JLPT N2
* JLPT N1
* Additional Dictionaries
* Multiple Study Profiles
* Local Backup System
* Mobile Companion App

---

# 20. Deliverables

Claude Code should generate:

1. Complete Architecture
2. SQLx Migrations
3. SQLite Schema
4. Rust Backend Design
5. React Frontend Design
6. Import Pipeline
7. FSRS Integration
8. MVP Implementation Plan
9. Feature Roadmap
10. Detailed Todo Tasks
11. Development Milestones
12. Testing Strategy
