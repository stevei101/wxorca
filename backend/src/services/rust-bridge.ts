/**
 * Rust Bridge - Interface to the Rust agent CLI
 *
 * This module provides the bridge between the TypeScript backend and
 * the Rust agent implementation. Initially uses subprocess communication,
 * can be upgraded to FFI via napi-rs later.
 */

export interface AgentResponse {
  session_id: string;
  agent_type: string;
  response: string;
  error?: string;
}

/**
 * Map frontend agent type IDs to CLI argument values
 */
function mapAgentType(agentType: string): string {
  const mapping: Record<string, string> = {
    "admin-setup": "admin-setup",
    usage: "usage",
    troubleshoot: "troubleshoot",
    "best-practices": "best-practices",
    docs: "docs",
  };
  return mapping[agentType] || agentType;
}

/**
 * Invoke a Rust agent via subprocess
 *
 * In production, this spawns the compiled Rust CLI.
 * In development, it uses a mock response.
 */
export async function invokeRustAgent(
  agentType: string,
  sessionId: string,
  message: string
): Promise<AgentResponse> {
  const cliAgentType = mapAgentType(agentType);

  // Check if we're in development mode (no compiled binary)
  const isDevelopment = process.env.NODE_ENV === "development" || true; // Always use mock for now

  if (isDevelopment) {
    return mockAgentResponse(cliAgentType, sessionId, message);
  }

  try {
    // Spawn the Rust CLI process
    const proc = Bun.spawn(
      [
        "./target/release/wxorca-cli",
        "--agent",
        cliAgentType,
        "--session",
        sessionId,
        "--message",
        message,
        "--format",
        "json",
      ],
      {
        cwd: process.cwd().replace("/backend", ""),
        stdout: "pipe",
        stderr: "pipe",
      }
    );

    // Read stdout
    const output = await new Response(proc.stdout).text();
    const stderr = await new Response(proc.stderr).text();

    // Wait for process to exit
    const exitCode = await proc.exited;

    if (exitCode !== 0) {
      console.error("CLI stderr:", stderr);
      return {
        session_id: sessionId,
        agent_type: agentType,
        response: "",
        error: `Agent process exited with code ${exitCode}: ${stderr}`,
      };
    }

    // Parse the JSON response
    try {
      return JSON.parse(output) as AgentResponse;
    } catch (parseError) {
      return {
        session_id: sessionId,
        agent_type: agentType,
        response: output,
        error: undefined,
      };
    }
  } catch (err) {
    console.error("Failed to spawn agent process:", err);
    return {
      session_id: sessionId,
      agent_type: agentType,
      response: "",
      error: `Failed to invoke agent: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
}

/**
 * Mock agent response for development
 */
function mockAgentResponse(
  agentType: string,
  sessionId: string,
  message: string
): AgentResponse {
  const responses: Record<string, (msg: string) => string> = {
    "admin-setup": (msg) => generateAdminResponse(msg),
    usage: (msg) => generateUsageResponse(msg),
    troubleshoot: (msg) => generateTroubleshootResponse(msg),
    "best-practices": (msg) => generateBestPracticesResponse(msg),
    docs: (msg) => generateDocsResponse(msg),
  };

  const responseGenerator = responses[agentType] || responses["usage"];

  return {
    session_id: sessionId,
    agent_type: agentType,
    response: responseGenerator(message),
  };
}

function generateAdminResponse(message: string): string {
  const msgLower = message.toLowerCase();

  if (msgLower.includes("setup") || msgLower.includes("install")) {
    return `## WatsonX Orchestrate Setup Guide

Here's how to set up WatsonX Orchestrate:

1. **Access the Admin Console**: Navigate to your WXO instance and log in with admin credentials.

2. **Configure Identity Provider**: Set up SSO or local authentication under Settings > Security.

3. **Create User Groups**: Define roles and permissions in Settings > Users & Teams.

4. **Set Up Integrations**: Connect external services in Settings > Integrations.

Would you like detailed steps for any of these areas?`;
  }

  if (msgLower.includes("user") || msgLower.includes("permission")) {
    return `## User Management

To manage users in WatsonX Orchestrate:

1. Go to **Settings > Users & Teams**
2. Click **Add User** to invite new users
3. Assign appropriate roles (Admin, Developer, User)
4. Configure team memberships for collaboration

**Tip**: Use groups to manage permissions at scale.

Need help with a specific user management task?`;
  }

  return `I'm the Admin Setup Guide for WatsonX Orchestrate.

I can help you with:
- Initial setup and configuration
- User and team management
- Security settings
- Integration configuration
- API key management

What would you like help with?`;
}

function generateUsageResponse(message: string): string {
  const msgLower = message.toLowerCase();

  if (msgLower.includes("skill")) {
    return `## Working with Skills

Skills are the building blocks of WatsonX Orchestrate. Here's how to work with them:

### Creating a Skill
1. Click **+ New Skill** in the skill catalog
2. Choose a skill type (API, Custom, Pre-built)
3. Define inputs and outputs
4. Test your skill before publishing

### Using Skills
- Type naturally: "Send an email to John about the meeting"
- WXO will find and execute the right skill
- Review and confirm before execution

Would you like more details on creating or using skills?`;
  }

  if (msgLower.includes("workflow") || msgLower.includes("automation")) {
    return `## Building Workflows

Workflows let you chain skills together for complex automations:

1. **Design**: Map out the steps in your process
2. **Build**: Add skills to your workflow canvas
3. **Connect**: Define data flow between steps
4. **Test**: Run the workflow with test data
5. **Deploy**: Publish for your team to use

**💡 Pro Tip**: Start simple and add complexity gradually.

What kind of workflow are you trying to build?`;
  }

  return `## Getting Started with WatsonX Orchestrate

Welcome! I can help you with:

- **Skills**: Creating and using automation skills
- **Workflows**: Building multi-step automations
- **Catalog**: Finding pre-built integrations
- **AI Features**: Natural language interaction

What would you like to learn about?`;
}

function generateTroubleshootResponse(message: string): string {
  const msgLower = message.toLowerCase();

  if (
    msgLower.includes("login") ||
    msgLower.includes("auth") ||
    msgLower.includes("access")
  ) {
    return `## 🔍 Issue Analysis: AUTHENTICATION

**Severity**: 🔴 High

### Likely Causes
- Expired credentials or tokens
- Incorrect SSO configuration
- User permissions not set correctly
- API key revoked or expired

### Troubleshooting Steps

1. Verify credentials are correct
2. Check token expiration
3. Review user permissions
4. Test SSO configuration

### Quick Fix Attempts
1. Clear browser cache and cookies
2. Try logging out and back in
3. Check if your session has expired
4. Verify your account is active

**⚠️ If issues persist**, contact your administrator to verify your account permissions.

Can you tell me more about the specific error you're seeing?`;
  }

  if (msgLower.includes("slow") || msgLower.includes("performance")) {
    return `## 🔍 Issue Analysis: PERFORMANCE

**Severity**: 🟡 Medium

### Likely Causes
- High system load
- Network latency
- Large data volumes
- Resource constraints

### Troubleshooting Steps

1. Check system status page
2. Monitor network connectivity
3. Review workflow complexity
4. Check concurrent user count

### Quick Fix Attempts
1. Refresh the page
2. Check your internet connection
3. Try a different browser
4. Check the WXO status page for outages

**💡 Tip**: If working with large datasets, try processing in smaller batches.`;
  }

  return `## 🔧 Troubleshooting Assistant

I can help you diagnose and resolve issues with WatsonX Orchestrate.

Please describe your issue, including:
- What exactly happened?
- Any error messages shown?
- When did this start?
- Any recent changes?

Common issues I can help with:
- Authentication and login problems
- Skill execution failures
- Integration connection issues
- Performance problems`;
}

function generateBestPracticesResponse(message: string): string {
  const msgLower = message.toLowerCase();

  if (msgLower.includes("workflow")) {
    return `## 🏆 Best Practices: WORKFLOW DESIGN

### Workflow Design Principles

**1. Keep it Modular**
- Break complex workflows into reusable sub-workflows
- Each workflow should do one thing well
- Use consistent naming conventions

**2. Plan for Failure**
- Add error handling at each critical step
- Use retries with exponential backoff
- Log failures for debugging

**3. Document Everything**
- Add descriptions to workflows and steps
- Document expected inputs and outputs
- Maintain a changelog

**4. Test Thoroughly**
- Test with edge cases
- Use staging environments
- Validate before production deployment

---

**💡 Need more specific advice?** Tell me about your use case and I can provide tailored recommendations.`;
  }

  if (msgLower.includes("security")) {
    return `## 🏆 Best Practices: SECURITY

### Security Best Practices

**1. Access Control**
- Follow the principle of least privilege
- Review permissions regularly
- Use role-based access control (RBAC)

**2. Credential Management**
- Never hardcode credentials
- Use secure secret storage
- Rotate credentials regularly

**3. Data Protection**
- Encrypt sensitive data in transit and at rest
- Minimize data retention
- Audit data access

**4. Monitoring & Compliance**
- Enable audit logging
- Set up security alerts
- Conduct regular security reviews

What specific security aspect would you like guidance on?`;
  }

  return `## 🏆 Best Practices Coach

I can help you optimize your use of WatsonX Orchestrate. I provide guidance on:

- **Workflow Design**: Building efficient, maintainable workflows
- **Performance**: Optimizing for speed and reliability
- **Security**: Protecting your data and access
- **Skill Development**: Creating reusable, well-designed skills
- **Team Collaboration**: Working effectively with others

What area would you like best practices guidance on?`;
}

function generateDocsResponse(message: string): string {
  const msgLower = message.toLowerCase();

  if (msgLower.includes("api")) {
    return `## 📚 Documentation Guide

### API Documentation

The WatsonX Orchestrate API documentation covers:

- **Authentication**: How to obtain and use API tokens
- **Skills API**: Create, manage, and execute skills
- **Workflows API**: Manage workflow definitions
- **Users API**: User and team management

**Quick Links:**
- [API Reference](https://www.ibm.com/docs/watsonx-orchestrate/api)
- [Authentication Guide](https://www.ibm.com/docs/watsonx-orchestrate/api/auth)

Would you like me to explain a specific API endpoint?`;
  }

  if (msgLower.includes("start") || msgLower.includes("begin")) {
    return `## 📚 Documentation Guide

### Getting Started

Welcome to WatsonX Orchestrate! Here's how to begin:

1. **First Steps**: Log in and explore the interface
2. **Try a Skill**: Use a pre-built skill from the catalog
3. **Create Your Own**: Build a simple custom skill
4. **Automate**: Combine skills into workflows

**Quick Links:**
- [Quick Start Guide](https://www.ibm.com/docs/watsonx-orchestrate/quickstart)
- [Tutorial Videos](https://www.ibm.com/docs/watsonx-orchestrate/tutorials)

What specific topic would you like to learn about first?`;
  }

  return `## 📚 Documentation Helper

I can help you find and understand WatsonX Orchestrate documentation.

**Documentation Categories:**
- **Getting Started**: Onboarding and first steps
- **User Guide**: Daily usage and features
- **Admin Guide**: Setup and configuration
- **API Reference**: Technical integration details
- **Troubleshooting**: Common issues and solutions

What documentation are you looking for?`;
}
