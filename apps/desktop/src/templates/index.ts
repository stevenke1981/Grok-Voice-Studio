export type TemplateCategory =
  | "horror"
  | "daily"
  | "shortvideo"
  | "news"
  | "fairy"
  | "comic"
  | "english"
  | "narration";

export interface ScriptTemplate {
  id: string;
  name: string;
  nameEn: string;
  category: TemplateCategory;
  description: string;
  descriptionEn: string;
  content: string;
}

export interface StoryTemplate {
  id: string;
  name: string;
  nameEn: string;
  style: string;
  description: string;
  descriptionEn: string;
  content: string;
}

export const DIALOGUE_TEMPLATES: ScriptTemplate[] = [
  {
    id: "horror-night",
    name: "恐怖懸疑 · 雨夜",
    nameEn: "Horror · Rainy Night",
    category: "horror",
    description: "4 角色、懸疑氛圍、含 speech tags",
    descriptionEn: "4 characters, suspense, with speech tags",
    content: `旁白：深夜的城市，只剩雨聲。
音效：雨聲
阿明（緊張）：你聽到了嗎？{雷聲} [pause] 那不是風聲。
小雅（笑）：別自己嚇自己。
怪物（低沉）：你們終於來了。`,
  },
  {
    id: "daily-friends",
    name: "日常對話 · 朋友出遊",
    nameEn: "Daily · Friends Trip",
    category: "daily",
    description: "3 角色輕鬆對話",
    descriptionEn: "3 characters, casual chat",
    content: `旁白：週末早晨，陽光灑進咖啡廳。
小明：今天天氣真好，我們去海邊吧！
小美（興奮）：好啊！我記得附近有家超好吃的冰店。
老闆（親切）：兩位，要外帶還是內用？`,
  },
  {
    id: "shortvideo-hook",
    name: "短影音 · 開場 Hook",
    nameEn: "Short Video · Hook",
    category: "shortvideo",
    description: "旁白 + 角色，適合 Reels / Shorts",
    descriptionEn: "Narrator + character, for short-form video",
    content: `旁白：你知道嗎？這座城市底下，藏著一座從未被人發現的地鐵站。
主持人（熱情）：今天，我們要帶你走進傳說中的「幽靈月台」。[pause]
路人（疑惑）：這裡真的有人來過嗎？
旁白：而答案，可能比你想象的更可怕。`,
  },
  {
    id: "news-broadcast",
    name: "新聞播報",
    nameEn: "News Broadcast",
    category: "news",
    description: "主播 + 記者連線格式",
    descriptionEn: "Anchor + field reporter format",
    content: `主播：各位觀眾晚安，歡迎收看今晚的新聞快報。
主播：首先帶您關注的是，南部地區今晚迎來今年最大規模的雷雨。
記者（現場）：我現在在現場，可以看到路面积水已經超過十公分。[sigh]
主播：請問目前民眾需要特別注意什麼？
記者：建議民眾避免外出，並留意氣象局最新警報。`,
  },
  {
    id: "fairy-tale",
    name: "童話 · 森林冒險",
    nameEn: "Fairy Tale · Forest",
    category: "fairy",
    description: "旁白 + 童話角色",
    descriptionEn: "Narrator + fairy tale characters",
    content: `旁白：很久很久以前，在一片會發光的森林裡，住著一隻會說話的狐狸。
小紅帽（天真）：狐狸先生，請問通往奶奶家的路怎麼走？
狐狸（狡猾）：呵呵，跟我來吧。[chuckle]
旁白：但小紅帽不知道，這條路將帶她走向一場意想不到的冒險。`,
  },
  {
    id: "comic-review",
    name: "漫畫解說",
    nameEn: "Comic Commentary",
    category: "comic",
    description: "解說 UP 主風格",
    descriptionEn: "Commentary / reviewer style",
    content: `旁白：第 47 話，作者終於揭開了十年伏筆。
UP主（激動）：各位！這一頁的分鏡簡直封神！[pause] 你們看到那個眼神了嗎？
搭檔（吐槽）：冷靜點，他只是眨了個眼。
UP主：不，這絕對是暗示主角黑化的關鍵！[breath]
旁白：而此時的讀者們，早已在留言區吵翻了。`,
  },
  {
    id: "english-cafe",
    name: "English · Café Scene",
    nameEn: "English · Café Scene",
    category: "english",
    description: "English dialogue, 3 characters",
    descriptionEn: "English dialogue sample",
    content: `Narrator: A quiet afternoon at a corner café in Brooklyn.
Emma: I can't believe you're moving to Tokyo next month.
Jack (sad): Yeah... it's hard to say goodbye.
Barista: Can I get you two another latte?
Emma: [sigh] Just one more minute, please.`,
  },
  {
    id: "narration-doc",
    name: "紀錄片旁白",
    nameEn: "Documentary Narration",
    category: "narration",
    description: "純旁白多段，適合紀錄片",
    descriptionEn: "Multi-part narrator, documentary style",
    content: `旁白：地球誕生至今四十六億年。
旁白：在這段漫長的歲月裡，生命曾數次瀕臨滅絕。[pause]
旁白：然而每一次，自然都以不可思議的方式重新書寫歷史。
旁白：今天，我們將跟隨鏡頭，走進那些倖存者的故事。`,
  },
];

export const STORY_TEMPLATES: StoryTemplate[] = [
  {
    id: "story-abandoned-school",
    name: "廢棄學校",
    nameEn: "Abandoned School",
    style: "恐怖",
    description: "懸疑短篇，適合 Story Mode",
    descriptionEn: "Suspense short story for Story Mode",
    content: `雷雨夜，廢棄學校裡只剩閃電照亮走廊。一名少女握緊手電筒，緩緩走向走廊盡頭。她聽見熟悉的腳步聲，卻看不見任何人影。

「有人在那裡嗎？」她顫聲問道。

黑暗中傳來低沉的回應：「你終於回來了。」`,
  },
  {
    id: "story-scifi",
    name: "科幻 · 最後的訊號",
    nameEn: "Sci-Fi · Last Signal",
    style: "一般敘事",
    description: "太空站科幻短篇",
    descriptionEn: "Space station sci-fi short",
    content: `公元 2187 年，深空探測站「曙光號」已與地球失聯三年。船長林薇在控制室發現了一則來自未知星系的訊號。訊號很短，只有三個字：「不要來。」

她望著窗外無盡的黑暗，不知道該轉發給地球，還是永遠藏起這個秘密。`,
  },
  {
    id: "story-fairy",
    name: "童話 · 會說話的種子",
    nameEn: "Fairy Tale · Talking Seed",
    style: "童話",
    description: "溫馨童話開頭",
    descriptionEn: "Warm fairy tale opening",
    content: `在雲霧繚繞的山谷裡，有一顆從不發芽的種子。農夫的女兒小禾每天都對它說話，告訴它外面的世界有多美麗。村民都笑她傻，直到某個春天的清晨，種子終於開口了：「謝謝你，我準備好了。」`,
  },
  {
    id: "story-shortvideo",
    name: "短影音 · 城市秘密",
    nameEn: "Short Video · City Secret",
    style: "短影音旁白",
    description: "適合短影音解說稿",
    descriptionEn: "Short-form video script source",
    content: `你以為你熟悉這座城市？其實在市中心地下三十公尺，有一條被封存半個世紀的地鐵隧道。官方記錄顯示它從未存在，但每隔十年，就會有人在月台邊緣聽見火車進站的聲音。而最近一次，是在三天前的午夜。`,
  },
  {
    id: "story-comic",
    name: "漫畫 · 轉生反派",
    nameEn: "Comic · Reborn Villain",
    style: "漫畫解說",
    description: "漫畫解說常用題材",
    descriptionEn: "Popular manhwa recap topic",
    content: `男主一覺醒來，發現自己穿進了最討厭的漫畫裡，而且還成了活不過三話的反派小弟。他記得所有劇情，卻改變不了即將到來的滅門慘案。除非，他能在主角抵達之前，先一步找到那個被作者遺忘的隱藏角色。`,
  },
];

export function getDialogueTemplate(id: string): ScriptTemplate | undefined {
  return DIALOGUE_TEMPLATES.find((t) => t.id === id);
}

export function getStoryTemplate(id: string): StoryTemplate | undefined {
  return STORY_TEMPLATES.find((t) => t.id === id);
}