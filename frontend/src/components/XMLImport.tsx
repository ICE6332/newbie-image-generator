import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";

interface Character {
  id: number;
  name: string;
  gender: string;
  appearance: string;
  clothing: string;
  expression: string;
  action: string;
  position: string;
}

interface GeneralTags {
  count: string;
  artists: string;
  style: string;
  background: string;
  lighting: string;
  atmosphere: string;
  objects: string;
  other: string;
}

export interface ParsedXML {
  characters: Character[];
  generalTags: GeneralTags;
  caption: string;
}

interface XMLImportProps {
  onImport: (data: ParsedXML) => void;
}

function parseTag(xml: string, tag: string): string {
  const match = xml.match(
    new RegExp(`<${tag}\\s*>([\\s\\S]*?)</${tag}\\s*>`, "i"),
  );
  return match ? match[1].trim() : "";
}

function createEmptyCharacter(id: number): Character {
  return {
    id,
    name: "",
    gender: "1girl",
    appearance: "",
    clothing: "",
    expression: "",
    action: "",
    position: "",
  };
}

export function parseXML(xml: string): ParsedXML {
  const characters: Character[] = [];

  // 1. 移除 <Prompt Start> 之前的系统提示词
  let content = xml;
  const promptStartMatch = xml.match(/<Prompt Start>[\s,]*/i);
  if (promptStartMatch) {
    content = xml.slice(promptStartMatch.index! + promptStartMatch[0].length);
  }

  // 解析角色 - 支持 <character_1> 和 <character 1> 两种格式
  const charMatches = content.matchAll(
    /<character[_ ](\d+)>([\s\S]*?)<\/character[_ ]\d+>/gi,
  );
  for (const match of charMatches) {
    const charXml = match[2];
    characters.push({
      id: parseInt(match[1]),
      name: parseTag(charXml, "n"),
      gender: parseTag(charXml, "gender"),
      appearance: parseTag(charXml, "appearance"),
      clothing: parseTag(charXml, "clothing"),
      expression: parseTag(charXml, "expression"),
      action: parseTag(charXml, "action"),
      position: parseTag(charXml, "position"),
    });
  }

  // 解析通用标签 - 支持 <general_tags> 和 <general tags> 两种格式
  let generalXml = parseTag(content, "general_tags");
  if (!generalXml) {
    const match = content.match(/<general tags>([\s\S]*?)<\/general tags>/i);
    generalXml = match ? match[1].trim() : "";
  }

  const generalTags: GeneralTags = {
    count: parseTag(generalXml, "count"),
    artists: parseTag(generalXml, "artists"),
    style: parseTag(generalXml, "style"),
    background: parseTag(generalXml, "background"),
    lighting: parseTag(generalXml, "lighting"),
    atmosphere: parseTag(generalXml, "atmosphere"),
    objects: parseTag(generalXml, "objects"),
    other: parseTag(generalXml, "other"),
  };

  // 2. 提取自然语言：移除所有已识别的结构化内容，剩余的就是自然语言
  let caption = content
    // 移除所有 character 块
    .replace(/<character[_ ]\d+>[\s\S]*?<\/character[_ ]\d+>/gi, "")
    // 移除 general_tags 块
    .replace(/<general_tags>[\s\S]*?<\/general_tags>/gi, "")
    .replace(/<general tags>[\s\S]*?<\/general tags>/gi, "")
    // 清理多余空白
    .trim();

  const captionFromTag = parseTag(content, "caption");
  const contentWithoutCaption = content.replace(
    /<caption[\s\S]*?>[\s\S]*?<\/caption>/gi,
    "",
  );

  caption = (captionFromTag || contentWithoutCaption)
    // 移除 <characters> 包裹层
    .replace(/<\/?characters[^>]*>/gi, "")
    // 移除所有 character 块
    .replace(/<character[_ ]\d+>[\s\S]*?<\/character[_ ]\d+>/gi, "")
    // 移除 general_tags 块
    .replace(/<general_tags>[\s\S]*?<\/general_tags>/gi, "")
    .replace(/<general tags>[\s\S]*?<\/general tags>/gi, "")
    .trim();

  return {
    characters: characters.length > 0 ? characters : [createEmptyCharacter(1)],
    generalTags,
    caption,
  };
}

export function XMLImport({ onImport }: XMLImportProps) {
  const [importText, setImportText] = useState("");
  const [isOpen, setIsOpen] = useState(false);

  const handleImport = () => {
    if (!importText.trim()) return;
    const parsed = parseXML(importText);
    onImport(parsed);
    setImportText("");
    setIsOpen(false);
  };

  return (
    <Card>
      <CardHeader
        className="py-3 cursor-pointer"
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm flex items-center gap-2">
            {isOpen ? "▼" : "▶"} 导入 XML Prompt
          </CardTitle>
          <Badge variant="outline" className="text-xs">
            {isOpen ? "点击收起" : "点击展开"}
          </Badge>
        </div>
      </CardHeader>
      {isOpen && (
        <CardContent className="pt-0 space-y-3">
          <Textarea
            value={importText}
            onChange={(e) => setImportText(e.target.value)}
            placeholder="粘贴 XML 格式的 prompt..."
            rows={4}
          />
          <Button
            size="sm"
            onClick={handleImport}
            disabled={!importText.trim()}
          >
            导入并填充
          </Button>
        </CardContent>
      )}
    </Card>
  );
}
