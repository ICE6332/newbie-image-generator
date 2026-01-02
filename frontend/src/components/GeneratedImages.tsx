import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { ImageResult } from "@/lib/types";
import { api } from "@/lib/api";

interface GeneratedImagesProps {
  images: ImageResult[];
  previewImage?: string | null;
}

export function GeneratedImages({
  images,
  previewImage,
}: GeneratedImagesProps) {
  const [selectedIndex, setSelectedIndex] = useState<number>(0);

  const getUrl = (image: ImageResult) =>
    api.getImageUrl(image.filename, image.subfolder, image.type);

  const handleDownload = async (image: ImageResult) => {
    try {
      const response = await fetch(getUrl(image));
      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = image.filename;
      link.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error("Download failed:", error);
    }
  };

  if (images.length === 0 && !previewImage) {
    return (
      <Card className="h-full">
        <CardContent className="flex flex-col items-center justify-center h-[600px] text-center">
          <div className="text-6xl mb-4">🎨</div>
          <h3 className="text-lg font-medium mb-2">No images generated yet</h3>
          <p className="text-muted-foreground">
            Enter a prompt on the left and click Generate to create images
          </p>
        </CardContent>
      </Card>
    );
  }

  const currentImage = images[selectedIndex];

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="shrink-0">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Generated Images</CardTitle>
            <CardDescription>Total: {images.length} images</CardDescription>
          </div>
          <Badge variant="secondary">{images.length}</Badge>
        </div>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col gap-4 overflow-hidden">
        {/* Main preview area */}
        <div className="flex-1 relative rounded-lg overflow-hidden bg-muted border min-h-0">
          {previewImage ? (
            <img
              src={previewImage}
              alt="Live preview"
              className="w-full h-full object-contain"
            />
          ) : currentImage ? (
            <img
              src={getUrl(currentImage)}
              alt={`Generated image ${selectedIndex + 1}`}
              className="w-full h-full object-contain"
            />
          ) : null}

          {/* Download button overlay */}
          {currentImage && !previewImage && (
            <div className="absolute bottom-4 right-4 flex gap-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={() => window.open(getUrl(currentImage), "_blank")}
              >
                🔍 Full
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handleDownload(currentImage)}
              >
                ⬇️ Download
              </Button>
            </div>
          )}

          {previewImage && (
            <div className="absolute top-4 left-4">
              <Badge>Live Preview</Badge>
            </div>
          )}
        </div>

        {/* Thumbnail strip */}
        {images.length > 0 && (
          <div className="shrink-0 flex gap-2 overflow-x-auto pb-2">
            {images.map((image, index) => (
              <button
                key={`${image.filename}-${index}`}
                onClick={() => setSelectedIndex(index)}
                className={`shrink-0 w-16 h-16 rounded-md overflow-hidden border-2 transition-all ${
                  index === selectedIndex
                    ? "border-primary"
                    : "border-transparent hover:border-muted-foreground"
                }`}
              >
                <img
                  src={getUrl(image)}
                  alt={`Thumbnail ${index + 1}`}
                  className="w-full h-full object-cover"
                  loading="lazy"
                />
              </button>
            ))}
          </div>
        )}

        {/* Current image info */}
        {currentImage && !previewImage && (
          <div className="shrink-0 text-sm text-muted-foreground text-right">
            {currentImage.filename} • #{images.length - selectedIndex}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
