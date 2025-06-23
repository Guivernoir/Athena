import { InputForm } from "./InputForm";

export const WelcomeScreen = ({
  handleSubmit,
  onCancel,
  isLoading,
}) => (
  <div className="welcome-screen">
    <div className="welcome-content">
      <div className="welcome-header">
        <h1 className="welcome-title font-heading">
          Welcome.
        </h1>
        <p className="welcome-subtitle font-description">
          How can I help you today?
        </p>
      </div>
      
      <div className="welcome-form-container">
        <InputForm
          onSubmit={handleSubmit}
          isLoading={isLoading}
          onCancel={onCancel}
          hasHistory={false}
        />
      </div>
      
      <div className="welcome-footer">
        <p className="welcome-powered-by">
          Fluent in human and machine. Fluent in you.
        </p>
      </div>
    </div>
    
    {/* Decorative background elements */}
    <div className="welcome-bg-decoration">
      <div className="bg-orb bg-orb-1"></div>
      <div className="bg-orb bg-orb-2"></div>
      <div className="bg-orb bg-orb-3"></div>
    </div>
  </div>
);