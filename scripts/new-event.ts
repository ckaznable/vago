import fs from 'fs';
import path from 'path';

const CONTENT_DIR = path.resolve(process.cwd(), 'content');

function createNewEntry() {
  const now = new Date();
  const defaultDate = now.toISOString().split('T')[0]; // YYYY-MM-DD
  const defaultTime = `${now.getHours().toString().padStart(2, '0')}${now.getMinutes().toString().padStart(2, '0')}`; // HHmm

  const dateInput = process.argv[2] || defaultDate;
  const timeInput = process.argv[3] || defaultTime;

  const filename = `${dateInput}-${timeInput}.md`;
  const filePath = path.join(CONTENT_DIR, filename);

  if (!fs.existsSync(CONTENT_DIR)) {
    fs.mkdirSync(CONTENT_DIR, { recursive: true });
  }

  if (fs.existsSync(filePath)) {
    console.error(`[ERROR] File already exists: ${filename}`);
    process.exit(1);
  }

  const randomRec = Math.floor(Math.random() * (99 - 10 + 1) + 10);

  const template = `---
year: "${dateInput} ${timeInput.slice(0, 2)}:${timeInput.slice(2, 4)}"
title: "新事件標題"
dataRec: "${randomRec}%"
---
在這裡輸入歷史事件的描述內容。可以使用 **Markdown** 語法。
`;

  fs.writeFileSync(filePath, template, 'utf8');
  console.log(`[SUCCESS] Created new entry: ${filename}`);
}

createNewEntry();
