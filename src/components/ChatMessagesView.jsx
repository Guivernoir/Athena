import { ScrollArea } from "./ui/ScrollArea";
import { Loader2, Copy, CopyCheck } from "lucide-react";
import { InputForm } from "./InputForm";
import { Button } from "./ui/Button";
import { useState } from "react";
import ReactMarkdown from "react-markdown";
import { cn } from "./ui/utils";
import { Badge } from "./ui/Badge";
import { ActivityTimeline } from "./ActivityTimeline";

// Enhanced markdown components with design system styling
const mdComponents = {
  h1: ({ className, children, ...props }) => (
    <h1 className={cn("markdown-heading markdown-h1", className)} {...props}>
      {children}
    </h1>
  ),
  h2: ({ className, children, ...props }) => (
    <h2 className={cn("markdown-heading markdown-h2", className)} {...props}>
      {children}
    </h2>
  ),
  h3: ({ className, children, ...props }) => (
    <h3 className={cn("markdown-heading markdown-h3", className)} {...props}>
      {children}
    </h3>
  ),
  p: ({ className, children, ...props }) => (
    <p className={cn("markdown-paragraph", className)} {...props}>
      {children}
    </p>
  ),
  a: ({ className, children, href, ...props }) => (
    <Badge className="markdown-link-badge">
      <a
        className={cn("markdown-link", className)}
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        {...props}
      >
        {children}
      </a>
    </Badge>
  ),
  ul: ({ className, children, ...props }) => (
    <ul className={cn("markdown-list markdown-unordered-list", className)} {...props}>
      {children}
    </ul>
  ),
  ol: ({ className, children, ...props }) => (
    <ol className={cn("markdown-list markdown-ordered-list", className)} {...props}>
      {children}
    </ol>
  ),
  li: ({ className, children, ...props }) => (
    <li className={cn("markdown-list-item", className)} {...props}>
      {children}
    </li>
  ),
  blockquote: ({ className, children, ...props }) => (
    <blockquote className={cn("markdown-blockquote", className)} {...props}>
      {children}
    </blockquote>
  ),
  code: ({ className, children, ...props }) => (
    <code className={cn("markdown-code-inline", className)} {...props}>
      {children}
    </code>
  ),
  pre: ({ className, children, ...props }) => (
    <pre className={cn("markdown-code-block", className)} {...props}>
      {children}
    </pre>
  ),
  hr: ({ className, ...props }) => (
    <hr className={cn("markdown-divider", className)} {...props} />
  ),
  table: ({ className, children, ...props }) => (
    <div className="markdown-table-wrapper">
      <table className={cn("markdown-table", className)} {...props}>
        {children}
      </table>
    </div>
  ),
  th: ({ className, children, ...props }) => (
    <th className={cn("markdown-table-header", className)} {...props}>
      {children}
    </th>
  ),
  td: ({ className, children, ...props }) => (
    <td className={cn("markdown-table-cell", className)} {...props}>
      {children}
    </td>
  ),
};

// HumanMessageBubble Component
const HumanMessageBubble = ({ message, mdComponents }) => {
  return (
    <div className="message-bubble human-message">
      <ReactMarkdown components={mdComponents}>
        {typeof message.content === "string"
          ? message.content
          : JSON.stringify(message.content)}
      </ReactMarkdown>
    </div>
  );
};

// AiMessageBubble Component
const AiMessageBubble = ({
  message,
  historicalActivity,
  liveActivity,
  isLastMessage,
  isOverallLoading,
  mdComponents,
  handleCopy,
  copiedMessageId,
}) => {
  // Determine which activity events to show and if it's for a live loading message
  const activityForThisBubble =
    isLastMessage && isOverallLoading ? liveActivity : historicalActivity;
  const isLiveActivityForThisBubble = isLastMessage && isOverallLoading;

  return (
    <div className="message-bubble ai-message">
      {activityForThisBubble && activityForThisBubble.length > 0 && (
        <div className="activity-timeline-container">
          <ActivityTimeline
            processedEvents={activityForThisBubble}
            isLoading={isLiveActivityForThisBubble}
          />
        </div>
      )}
      
      <div className="message-content">
        <ReactMarkdown components={mdComponents}>
          {typeof message.content === "string"
            ? message.content
            : JSON.stringify(message.content)}
        </ReactMarkdown>
      </div>
      
      <Button
        variant="default"
        className={`copy-button ${message.content.length > 0 ? "visible" : "hidden"}`}
        onClick={() =>
          handleCopy(
            typeof message.content === "string"
              ? message.content
              : JSON.stringify(message.content),
            message.id
          )
        }
      >
        <span className="copy-button-text">
          {copiedMessageId === message.id ? "Copied" : "Copy"}
        </span>
        <span className="copy-button-icon">
          {copiedMessageId === message.id ? <CopyCheck /> : <Copy />}
        </span>
      </Button>
    </div>
  );
};

// LoadingBubble Component
const LoadingBubble = ({ liveActivityEvents }) => {
  return (
    <div className="message-bubble ai-message loading-bubble">
      {liveActivityEvents.length > 0 ? (
        <div className="activity-timeline-container">
          <ActivityTimeline
            processedEvents={liveActivityEvents}
            isLoading={true}
          />
        </div>
      ) : (
        <div className="loading-content">
          <Loader2 className="loading-spinner" />
          <span className="loading-text">Processing...</span>
        </div>
      )}
    </div>
  );
};

export function ChatMessagesView({
  messages,
  isLoading,
  scrollAreaRef,
  onSubmit,
  onCancel,
  liveActivityEvents,
  historicalActivities,
}) {
  const [copiedMessageId, setCopiedMessageId] = useState(null);

  const handleCopy = async (text, messageId) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedMessageId(messageId);
      setTimeout(() => setCopiedMessageId(null), 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };
  
  return (
    <div className="chat-messages-container">
      <ScrollArea className="chat-scroll-area" ref={scrollAreaRef}>
        <div className="chat-messages">
          {messages.map((message, index) => {
            const isLast = index === messages.length - 1;
            return (
              <div key={message.id || `msg-${index}`} className="message-wrapper">
                <div className={`message-row ${message.type === "human" ? "human-row" : "ai-row"}`}>
                  {message.type === "human" ? (
                    <HumanMessageBubble
                      message={message}
                      mdComponents={mdComponents}
                    />
                  ) : (
                    <AiMessageBubble
                      message={message}
                      historicalActivity={historicalActivities[message.id]}
                      liveActivity={liveActivityEvents}
                      isLastMessage={isLast}
                      isOverallLoading={isLoading}
                      mdComponents={mdComponents}
                      handleCopy={handleCopy}
                      copiedMessageId={copiedMessageId}
                    />
                  )}
                </div>
              </div>
            );
          })}
          
          {isLoading &&
            (messages.length === 0 ||
              messages[messages.length - 1].type === "human") && (
              <div className="message-wrapper">
                <div className="message-row ai-row">
                  <LoadingBubble liveActivityEvents={liveActivityEvents} />
                </div>
              </div>
            )}
        </div>
      </ScrollArea>
      
      <InputForm
        onSubmit={onSubmit}
        isLoading={isLoading}
        onCancel={onCancel}
        hasHistory={messages.length > 0}
      />
    </div>
  );
}