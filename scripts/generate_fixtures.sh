#!/bin/bash

# Generate fixture data for GRIMOIRE
# Creates 200 items across all categories

# Detect OS and set appropriate database path
case "$(uname -s)" in
    Darwin)
        DB_PATH="${HOME}/Library/Application Support/grimoire/grimoire.db"
        ;;
    Linux)
        DB_PATH="${HOME}/.local/share/grimoire/grimoire.db"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        DB_PATH="${APPDATA}/grimoire/grimoire.db"
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "Database not found at $DB_PATH"
    echo "Please run the application first to create the database."
    exit 1
fi

echo "Generating fixture data..."

# Arrays for generating varied content
PROMPT_NAMES=(
    "code-review-template" "bug-report-format" "api-documentation"
    "commit-message-guide" "pr-description" "error-analysis"
    "refactoring-plan" "test-case-template" "changelog-entry"
    "readme-section" "api-endpoint-spec" "database-schema"
    "architecture-doc" "deployment-checklist" "security-audit"
    "performance-report" "user-story" "acceptance-criteria"
    "technical-spec" "design-rationale" "code-comment-style"
    "logging-format" "config-template" "env-example"
    "docker-compose-base" "ci-workflow" "release-notes"
    "migration-guide" "troubleshooting-guide" "faq-template"
    "onboarding-doc" "style-guide" "naming-conventions"
    "api-versioning" "error-codes" "status-messages"
    "validation-rules" "data-format" "response-template"
    "request-template" "webhook-payload" "event-schema"
    "notification-template" "email-template" "alert-format"
    "metric-definition" "sla-template" "runbook-template"
)

AGENT_NAMES=(
    "code-reviewer" "bug-hunter" "refactorer" "test-writer"
    "doc-generator" "security-analyst" "performance-optimizer"
    "api-designer" "database-expert" "devops-assistant"
    "frontend-specialist" "backend-architect" "fullstack-helper"
    "rust-expert" "python-guru" "typescript-wizard"
    "react-specialist" "vue-expert" "angular-helper"
    "node-assistant" "go-expert" "java-specialist"
    "kubernetes-ops" "aws-architect" "gcp-specialist"
    "azure-expert" "terraform-helper" "ansible-assistant"
    "git-expert" "ci-cd-specialist" "monitoring-guru"
    "logging-expert" "caching-specialist" "queue-expert"
    "graphql-designer" "rest-architect" "grpc-specialist"
    "websocket-expert" "auth-specialist" "oauth-helper"
    "jwt-expert" "encryption-specialist" "compliance-checker"
    "accessibility-auditor" "i18n-specialist" "seo-optimizer"
    "mobile-expert" "pwa-specialist" "electron-helper"
)

SKILL_NAMES=(
    "code-analysis" "syntax-check" "type-inference"
    "dependency-scan" "vulnerability-check" "license-audit"
    "complexity-analysis" "coverage-report" "lint-check"
    "format-code" "import-organizer" "dead-code-finder"
    "duplicate-detector" "memory-profiler" "cpu-profiler"
    "network-analyzer" "api-tester" "load-tester"
    "unit-test-gen" "integration-test-gen" "e2e-test-gen"
    "mock-generator" "fixture-creator" "snapshot-tester"
    "schema-validator" "json-formatter" "yaml-parser"
    "xml-processor" "csv-handler" "markdown-renderer"
    "html-sanitizer" "css-optimizer" "js-minifier"
    "image-optimizer" "font-subsetter" "svg-cleaner"
    "log-parser" "metric-collector" "trace-analyzer"
    "error-aggregator" "alert-router" "incident-creator"
    "changelog-gen" "release-noter" "version-bumper"
    "tag-creator" "branch-manager" "pr-automator"
    "issue-linker" "commit-analyzer" "blame-helper"
)

COMMAND_NAMES=(
    "review-pr" "check-branch" "run-tests" "deploy-staging"
    "deploy-prod" "rollback" "scale-up" "scale-down"
    "check-logs" "tail-errors" "search-logs" "export-metrics"
    "create-migration" "run-migration" "seed-database" "backup-db"
    "restore-db" "clear-cache" "warm-cache" "invalidate-cdn"
    "restart-service" "stop-service" "start-service" "health-check"
    "ssl-renew" "cert-check" "dns-lookup" "port-scan"
    "benchmark-api" "load-test" "stress-test" "chaos-test"
    "generate-types" "sync-schema" "validate-openapi" "generate-client"
    "build-docker" "push-image" "pull-image" "prune-images"
    "kubectl-apply" "helm-upgrade" "terraform-plan" "terraform-apply"
    "aws-sync" "gcp-deploy" "azure-publish" "vercel-deploy"
    "npm-audit" "yarn-upgrade" "cargo-update" "pip-check"
    "lint-all" "format-all" "test-all" "build-all"
)

TAGS=(
    "rust" "python" "typescript" "javascript" "go" "java"
    "react" "vue" "angular" "svelte" "nextjs" "nuxt"
    "api" "rest" "graphql" "grpc" "websocket"
    "database" "sql" "nosql" "redis" "postgres" "mongodb"
    "docker" "kubernetes" "aws" "gcp" "azure" "terraform"
    "testing" "ci-cd" "devops" "security" "performance"
    "frontend" "backend" "fullstack" "mobile" "desktop"
    "documentation" "review" "refactoring" "debugging"
)

MODELS=("sonnet" "opus" "haiku" "gpt-4o" "gpt-4o-mini" "")

TOOLS_AGENT=("Read,Grep,Glob" "Read,Write,Edit" "Bash,Read,Grep" "Read,Grep,Glob,Edit" "Bash,Read,Write" "")

TOOLS_SKILL=("Read,Grep" "Read,Glob" "Grep,Glob" "Read" "")

TOOLS_COMMAND=('Bash(git:*)' 'Bash(npm:*),Bash(yarn:*)' 'Bash(docker:*)' 'Bash(kubectl:*)' 'Read,Grep' '')

# Function to get random element from array
random_element() {
    local arr=("$@")
    echo "${arr[$RANDOM % ${#arr[@]}]}"
}

# Function to get random tags (1-3 tags)
random_tags() {
    local num_tags=$((1 + RANDOM % 3))
    local selected=""
    for ((i=0; i<num_tags; i++)); do
        local tag=$(random_element "${TAGS[@]}")
        if [ -z "$selected" ]; then
            selected="$tag"
        else
            selected="$selected,$tag"
        fi
    done
    echo "$selected"
}

# Function to escape single quotes for SQL
escape_sql() {
    echo "$1" | sed "s/'/''/g"
}

# Generate items
count=0

# Generate 50 Prompts
echo "Generating prompts..."
for name in "${PROMPT_NAMES[@]}"; do
    tags=$(random_tags)
    desc="Template for $(echo $name | tr '-' ' ')"
    content="# $(echo $name | tr '-' ' ' | sed 's/\b\(.\)/\u\1/g')

## Purpose
This template helps with $(echo $name | tr '-' ' ').

## Usage
1. Copy this template
2. Fill in the relevant sections
3. Customize as needed

## Template

\`\`\`
[Your content here]
\`\`\`

## Notes
- Keep it concise
- Follow the established patterns
- Update as requirements change"

    sqlite3 "$DB_PATH" "INSERT OR IGNORE INTO items (name, category, description, content, tags, created_at, updated_at)
        VALUES ('$(escape_sql "$name")', 'prompt', '$(escape_sql "$desc")', '$(escape_sql "$content")', '$(escape_sql "$tags")',
        datetime('now', '-' || abs(random() % 30) || ' days'), datetime('now', '-' || abs(random() % 7) || ' days'));"
    ((count++))
done

# Generate 50 Agents
echo "Generating agents..."
for name in "${AGENT_NAMES[@]}"; do
    tags=$(random_tags)
    model=$(random_element "${MODELS[@]}")
    tools=$(random_element "${TOOLS_AGENT[@]}")
    desc="AI assistant specialized in $(echo $name | tr '-' ' ')"
    content="You are an expert $(echo $name | tr '-' ' ').

## Capabilities
- Deep understanding of the domain
- Best practices and patterns
- Code review and suggestions
- Problem solving

## Guidelines
1. Always explain your reasoning
2. Provide concrete examples
3. Consider edge cases
4. Suggest improvements proactively

## Response Format
- Start with a brief assessment
- Provide detailed analysis
- End with actionable recommendations"

    model_sql=""
    if [ -n "$model" ]; then
        model_sql="'$model'"
    else
        model_sql="NULL"
    fi

    tools_sql=""
    if [ -n "$tools" ]; then
        tools_sql="'$tools'"
    else
        tools_sql="NULL"
    fi

    sqlite3 "$DB_PATH" "INSERT OR IGNORE INTO items (name, category, description, content, tags, model, tools, created_at, updated_at)
        VALUES ('$(escape_sql "$name")', 'agent', '$(escape_sql "$desc")', '$(escape_sql "$content")', '$(escape_sql "$tags")',
        $model_sql, $tools_sql,
        datetime('now', '-' || abs(random() % 30) || ' days'), datetime('now', '-' || abs(random() % 7) || ' days'));"
    ((count++))
done

# Generate 50 Skills
echo "Generating skills..."
for name in "${SKILL_NAMES[@]}"; do
    tags=$(random_tags)
    tools=$(random_element "${TOOLS_SKILL[@]}")
    desc="Skill for $(echo $name | tr '-' ' ')"
    content="# $(echo $name | tr '-' ' ' | sed 's/\b\(.\)/\u\1/g')

## Description
This skill performs $(echo $name | tr '-' ' ') operations.

## Instructions
1. Analyze the input
2. Apply the appropriate transformations
3. Return structured results

## Input Format
Accepts code files or text content.

## Output Format
Returns analysis results in a structured format."

    tools_sql=""
    if [ -n "$tools" ]; then
        tools_sql="'$tools'"
    else
        tools_sql="NULL"
    fi

    sqlite3 "$DB_PATH" "INSERT OR IGNORE INTO items (name, category, description, content, tags, allowed_tools, created_at, updated_at)
        VALUES ('$(escape_sql "$name")', 'skill', '$(escape_sql "$desc")', '$(escape_sql "$content")', '$(escape_sql "$tags")',
        $tools_sql,
        datetime('now', '-' || abs(random() % 30) || ' days'), datetime('now', '-' || abs(random() % 7) || ' days'));"
    ((count++))
done

# Generate 50 Commands
echo "Generating commands..."
for name in "${COMMAND_NAMES[@]}"; do
    tags=$(random_tags)
    tools=$(random_element "${TOOLS_COMMAND[@]}")
    model=$(random_element "${MODELS[@]}")
    desc="Command to $(echo $name | tr '-' ' ')"
    hint="[target]"
    content="Execute $(echo $name | tr '-' ' ') for the specified target.

## Steps
1. Validate the input parameters
2. Check prerequisites
3. Execute the main operation
4. Verify the results
5. Report status

## Arguments
- \$ARGUMENTS: The target to operate on

## Error Handling
- Check for common failure modes
- Provide clear error messages
- Suggest remediation steps"

    tools_sql=""
    if [ -n "$tools" ]; then
        tools_sql="'$tools'"
    else
        tools_sql="NULL"
    fi

    model_sql=""
    if [ -n "$model" ]; then
        model_sql="'$model'"
    else
        model_sql="NULL"
    fi

    sqlite3 "$DB_PATH" "INSERT OR IGNORE INTO items (name, category, description, content, tags, allowed_tools, argument_hint, model, created_at, updated_at)
        VALUES ('$(escape_sql "$name")', 'command', '$(escape_sql "$desc")', '$(escape_sql "$content")', '$(escape_sql "$tags")',
        $tools_sql, '$hint', $model_sql,
        datetime('now', '-' || abs(random() % 30) || ' days'), datetime('now', '-' || abs(random() % 7) || ' days'));"
    ((count++))
done

echo "Generated $count items."
echo "Done!"
