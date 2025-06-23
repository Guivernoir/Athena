import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
} from "./ui/Card";
import { ScrollArea } from "./ui/ScrollArea";
import {
  Loader2,
  Activity,
  Info,
  Search,
  TextSearch,
  Brain,
  Pen,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { useEffect, useState } from "react";

export function ActivityTimeline({
  processedEvents,
  isLoading,
}) {
  const [isTimelineCollapsed, setIsTimelineCollapsed] = useState(false);
  
  const getEventIcon = (title, index) => {
    if (index === 0 && isLoading && processedEvents.length === 0) {
      return <Loader2 className="activity-timeline__icon activity-timeline__icon--loading" />;
    }
    if (title.toLowerCase().includes("generating")) {
      return <TextSearch className="activity-timeline__icon" />;
    } else if (title.toLowerCase().includes("thinking")) {
      return <Loader2 className="activity-timeline__icon activity-timeline__icon--loading" />;
    } else if (title.toLowerCase().includes("reflection")) {
      return <Brain className="activity-timeline__icon" />;
    } else if (title.toLowerCase().includes("research")) {
      return <Search className="activity-timeline__icon" />;
    } else if (title.toLowerCase().includes("finalizing")) {
      return <Pen className="activity-timeline__icon" />;
    }
    return <Activity className="activity-timeline__icon" />;
  };

  useEffect(() => {
    if (!isLoading && processedEvents.length !== 0) {
      setIsTimelineCollapsed(true);
    }
  }, [isLoading, processedEvents]);

  return (
    <Card className="activity-timeline">
      <CardHeader>
        <CardDescription className="activity-timeline__header">
          <div
            className="activity-timeline__toggle"
            onClick={() => setIsTimelineCollapsed(!isTimelineCollapsed)}
          >
            Research
            {isTimelineCollapsed ? (
              <ChevronDown className="activity-timeline__chevron" />
            ) : (
              <ChevronUp className="activity-timeline__chevron" />
            )}
          </div>
        </CardDescription>
      </CardHeader>
      {!isTimelineCollapsed && (
        <ScrollArea className="activity-timeline__scroll-area">
          <CardContent>
            {isLoading && processedEvents.length === 0 && (
              <div className="activity-timeline__event">
                <div className="activity-timeline__line activity-timeline__line--full" />
                <div className="activity-timeline__marker activity-timeline__marker--loading">
                  <Loader2 className="activity-timeline__marker-icon activity-timeline__marker-icon--loading" />
                </div>
                <div className="activity-timeline__content">
                  <p className="activity-timeline__title">
                    Searching...
                  </p>
                </div>
              </div>
            )}
            {processedEvents.length > 0 ? (
              <div className="activity-timeline__events">
                {processedEvents.map((eventItem, index) => (
                  <div key={index} className="activity-timeline__event">
                    {index < processedEvents.length - 1 ||
                    (isLoading && index === processedEvents.length - 1) ? (
                      <div className="activity-timeline__line" />
                    ) : null}
                    <div className="activity-timeline__marker">
                      {getEventIcon(eventItem.title, index)}
                    </div>
                    <div className="activity-timeline__content">
                      <p className="activity-timeline__title">
                        {eventItem.title}
                      </p>
                      <p className="activity-timeline__description">
                        {typeof eventItem.data === "string"
                          ? eventItem.data
                          : Array.isArray(eventItem.data)
                          ? eventItem.data.join(", ")
                          : JSON.stringify(eventItem.data)}
                      </p>
                    </div>
                  </div>
                ))}
                {isLoading && processedEvents.length > 0 && (
                  <div className="activity-timeline__event">
                    <div className="activity-timeline__marker activity-timeline__marker--loading">
                      <Loader2 className="activity-timeline__marker-icon activity-timeline__marker-icon--loading" />
                    </div>
                    <div className="activity-timeline__content">
                      <p className="activity-timeline__title">
                        Searching...
                      </p>
                    </div>
                  </div>
                )}
              </div>
            ) : !isLoading ? (
              <div className="activity-timeline__empty">
                <Info className="activity-timeline__empty-icon" />
                <p className="activity-timeline__empty-title">No activity to display.</p>
                <p className="activity-timeline__empty-subtitle">
                  Timeline will update during processing.
                </p>
              </div>
            ) : null}
          </CardContent>
        </ScrollArea>
      )}
    </Card>
  );
}