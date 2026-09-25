use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agent_metrics::AgentMetricsCollector;
use crate::execution_metrics::ExecutionMetricsCollector;
use crate::retrieval_metrics::RetrievalMetricsCollector;
use crate::token_metrics::TokenMetricsCollector;
use crate::validation_metrics::ValidationMetricsCollector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub project_name: String,
    pub project_version: String,
    pub executive_summary: ExecutiveSummary,
    pub execution_summary: ExecutionSummary,
    pub token_summary: TokenSummary,
    pub retrieval_summary: RetrievalSummary,
    pub agent_summary: AgentSummary,
    pub validation_summary: ValidationSummary,
    pub recommendations: Vec<Recommendation>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub total_tasks_executed: usize,
    pub success_rate: f64,
    pub total_tokens_consumed: usize,
    pub total_cost_usd: f64,
    pub average_task_duration_ms: f64,
    pub health_score: f64,
    pub critical_issues: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub cancelled_executions: usize,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub total_artifacts_created: usize,
    pub total_artifacts_modified: usize,
    pub total_tests_created: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
    pub average_tokens_per_execution: f64,
    pub top_models: Vec<(String, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalSummary {
    pub total_retrievals: usize,
    pub average_duration_ms: f64,
    pub average_results_per_query: f64,
    pub average_precision: Option<f64>,
    pub total_tokens_retrieved: usize,
    pub retrieval_type_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub total_agents: usize,
    pub total_tasks_completed: usize,
    pub overall_success_rate: f64,
    pub top_performers: Vec<(String, f64)>,
    pub total_tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_validations: usize,
    pub pass_rate: f64,
    pub total_findings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub validation_type_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Uuid,
    pub priority: RecommendationPriority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

pub struct AuditReportGenerator;

#[derive(Debug, Clone)]
pub struct AuditReportInput {
    pub project_name: String,
    pub project_version: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub execution_collector: ExecutionMetricsCollector,
    pub token_collector: TokenMetricsCollector,
    pub retrieval_collector: RetrievalMetricsCollector,
    pub agent_collector: AgentMetricsCollector,
    pub validation_collector: ValidationMetricsCollector,
}

impl AuditReportGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(&self, input: AuditReportInput) -> AuditReport {
        let execution_summary = self.generate_execution_summary(&input.execution_collector);
        let token_summary = self.generate_token_summary(&input.token_collector);
        let retrieval_summary = self.generate_retrieval_summary(&input.retrieval_collector);
        let agent_summary = self.generate_agent_summary(&input.agent_collector);
        let validation_summary = self.generate_validation_summary(&input.validation_collector);

        let executive_summary = self.generate_executive_summary(
            &execution_summary,
            &token_summary,
            &validation_summary,
        );

        let recommendations = self.generate_recommendations(
            &executive_summary,
            &execution_summary,
            &token_summary,
            &retrieval_summary,
            &agent_summary,
            &validation_summary,
        );

        AuditReport {
            id: Uuid::new_v4(),
            generated_at: Utc::now(),
            period_start: input.period_start,
            period_end: input.period_end,
            project_name: input.project_name,
            project_version: input.project_version,
            executive_summary,
            execution_summary,
            token_summary,
            retrieval_summary,
            agent_summary,
            validation_summary,
            recommendations,
            metadata: HashMap::new(),
        }
    }

    fn generate_execution_summary(
        &self,
        collector: &ExecutionMetricsCollector,
    ) -> ExecutionSummary {
        let all_metrics = collector.get_all_metrics();

        let total_executions = all_metrics.len();
        let successful_executions = collector.get_successful_metrics().len();
        let failed_executions = collector.get_failed_metrics().len();
        let cancelled_executions = all_metrics
            .iter()
            .filter(|m| m.status == crate::execution_metrics::ExecutionMetricsStatus::Cancelled)
            .count();

        let success_rate = collector.success_rate();
        let average_duration_ms = collector.average_duration_ms().unwrap_or(0.0);

        let total_artifacts_created = all_metrics.iter().map(|m| m.artifacts_created.len()).sum();
        let total_artifacts_modified = all_metrics.iter().map(|m| m.artifacts_modified.len()).sum();
        let total_tests_created = all_metrics.iter().map(|m| m.tests_created.len()).sum();

        ExecutionSummary {
            total_executions,
            successful_executions,
            failed_executions,
            cancelled_executions,
            success_rate,
            average_duration_ms,
            total_artifacts_created,
            total_artifacts_modified,
            total_tests_created,
        }
    }

    fn generate_token_summary(&self, collector: &TokenMetricsCollector) -> TokenSummary {
        let summary = collector.get_summary();

        let mut model_usage: Vec<(String, usize)> = summary.model_usage.into_iter().collect();
        model_usage.sort_by_key(|a| std::cmp::Reverse(a.1));
        model_usage.truncate(5);

        let average_tokens_per_execution = if summary.total_tokens > 0 {
            summary.total_tokens as f64 / collector.get_all_metrics().len() as f64
        } else {
            0.0
        };

        TokenSummary {
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            total_tokens: summary.total_tokens,
            total_cost_usd: summary.total_cost_usd,
            average_tokens_per_execution,
            top_models: model_usage,
        }
    }

    fn generate_retrieval_summary(
        &self,
        collector: &RetrievalMetricsCollector,
    ) -> RetrievalSummary {
        let summary = collector.get_summary();

        RetrievalSummary {
            total_retrievals: summary.total_retrievals,
            average_duration_ms: summary.average_duration_ms,
            average_results_per_query: summary.average_results_per_query,
            average_precision: summary.average_precision,
            total_tokens_retrieved: summary.total_tokens_retrieved,
            retrieval_type_distribution: summary.retrieval_type_counts,
        }
    }

    fn generate_agent_summary(&self, collector: &AgentMetricsCollector) -> AgentSummary {
        let summary = collector.get_summary();

        let mut top_performers: Vec<(String, f64)> = summary
            .agent_details
            .iter()
            .filter_map(|m| {
                m.performance_score
                    .map(|score| (m.agent_name.clone(), score))
            })
            .collect();
        top_performers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        top_performers.truncate(5);

        AgentSummary {
            total_agents: summary.total_agents,
            total_tasks_completed: summary.total_tasks_completed,
            overall_success_rate: summary.overall_success_rate,
            top_performers,
            total_tokens_used: summary.total_tokens_used,
        }
    }

    fn generate_validation_summary(
        &self,
        collector: &ValidationMetricsCollector,
    ) -> ValidationSummary {
        let summary = collector.get_summary();

        ValidationSummary {
            total_validations: summary.total_validations,
            pass_rate: summary.overall_pass_rate,
            total_findings: summary.total_findings,
            critical_findings: summary.critical_findings,
            high_findings: summary.high_findings,
            validation_type_distribution: summary.validation_type_counts,
        }
    }

    fn generate_executive_summary(
        &self,
        execution: &ExecutionSummary,
        token: &TokenSummary,
        validation: &ValidationSummary,
    ) -> ExecutiveSummary {
        let health_score = self.calculate_health_score(execution, validation);

        ExecutiveSummary {
            total_tasks_executed: execution.total_executions,
            success_rate: execution.success_rate,
            total_tokens_consumed: token.total_tokens,
            total_cost_usd: token.total_cost_usd,
            average_task_duration_ms: execution.average_duration_ms,
            health_score,
            critical_issues: validation.critical_findings,
            warnings: validation.high_findings,
        }
    }

    fn calculate_health_score(
        &self,
        execution: &ExecutionSummary,
        validation: &ValidationSummary,
    ) -> f64 {
        let execution_score = execution.success_rate * 0.4;
        let validation_score = validation.pass_rate * 0.3;
        let quality_score = if validation.critical_findings == 0 {
            0.3
        } else if validation.critical_findings <= 2 {
            0.2
        } else {
            0.1
        };

        execution_score + validation_score + quality_score
    }

    fn generate_recommendations(
        &self,
        executive: &ExecutiveSummary,
        _execution: &ExecutionSummary,
        token: &TokenSummary,
        retrieval: &RetrievalSummary,
        agent: &AgentSummary,
        validation: &ValidationSummary,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if executive.success_rate < 0.8 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                priority: RecommendationPriority::High,
                category: "execution".to_string(),
                title: "Improve Task Success Rate".to_string(),
                description: format!(
                    "Current success rate is {:.1}%. Target should be above 80%.",
                    executive.success_rate * 100.0
                ),
                impact: "Improved reliability and reduced rework".to_string(),
                effort: "Medium".to_string(),
            });
        }

        if validation.critical_findings > 0 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                priority: RecommendationPriority::Critical,
                category: "validation".to_string(),
                title: "Address Critical Findings".to_string(),
                description: format!(
                    "There are {} critical findings that need immediate attention.",
                    validation.critical_findings
                ),
                impact: "Reduced risk and improved quality".to_string(),
                effort: "High".to_string(),
            });
        }

        if token.average_tokens_per_execution > 10000.0 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                priority: RecommendationPriority::Medium,
                category: "token_optimization".to_string(),
                title: "Optimize Token Usage".to_string(),
                description: format!(
                    "Average token usage per execution is {:.0}. Consider optimizing context size.",
                    token.average_tokens_per_execution
                ),
                impact: "Reduced costs and improved efficiency".to_string(),
                effort: "Medium".to_string(),
            });
        }

        if retrieval.average_duration_ms > 1000.0 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                priority: RecommendationPriority::Medium,
                category: "retrieval".to_string(),
                title: "Optimize Retrieval Performance".to_string(),
                description: format!(
                    "Average retrieval duration is {:.0}ms. Consider caching or indexing improvements.",
                    retrieval.average_duration_ms
                ),
                impact: "Faster task execution".to_string(),
                effort: "Medium".to_string(),
            });
        }

        if agent.overall_success_rate < 0.7 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                priority: RecommendationPriority::High,
                category: "agent_performance".to_string(),
                title: "Review Agent Performance".to_string(),
                description: format!(
                    "Overall agent success rate is {:.1}%. Review agent configurations and capabilities.",
                    agent.overall_success_rate * 100.0
                ),
                impact: "Improved agent effectiveness".to_string(),
                effort: "High".to_string(),
            });
        }

        recommendations
    }

    pub fn export_to_json(&self, report: &AuditReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    pub fn export_to_markdown(&self, report: &AuditReport) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Audit Report: {}\n\n", report.project_name));
        md.push_str(&format!("**Version:** {}\n", report.project_version));
        md.push_str(&format!(
            "**Period:** {} to {}\n",
            report.period_start.format("%Y-%m-%d"),
            report.period_end.format("%Y-%m-%d")
        ));
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!(
            "- **Health Score:** {:.1}/10\n",
            report.executive_summary.health_score * 10.0
        ));
        md.push_str(&format!(
            "- **Tasks Executed:** {}\n",
            report.executive_summary.total_tasks_executed
        ));
        md.push_str(&format!(
            "- **Success Rate:** {:.1}%\n",
            report.executive_summary.success_rate * 100.0
        ));
        md.push_str(&format!(
            "- **Total Tokens:** {}\n",
            report.executive_summary.total_tokens_consumed
        ));
        md.push_str(&format!(
            "- **Total Cost:** ${:.2}\n",
            report.executive_summary.total_cost_usd
        ));
        md.push_str(&format!(
            "- **Critical Issues:** {}\n",
            report.executive_summary.critical_issues
        ));
        md.push_str(&format!(
            "- **Warnings:** {}\n\n",
            report.executive_summary.warnings
        ));

        if !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                md.push_str(&format!("### {}. {}\n", i + 1, rec.title));
                md.push_str(&format!("**Priority:** {:?}\n\n", rec.priority));
                md.push_str(&format!("{}\n\n", rec.description));
                md.push_str(&format!("- **Impact:** {}\n", rec.impact));
                md.push_str(&format!("- **Effort:** {}\n\n", rec.effort));
            }
        }

        md
    }
}

impl Default for AuditReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_report_generator_creation() {
        let generator = AuditReportGenerator::new();
        let _ = generator;
    }

    #[test]
    fn test_generate_empty_report() {
        let generator = AuditReportGenerator::new();
        let input = AuditReportInput {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            period_start: Utc::now() - chrono::Duration::days(7),
            period_end: Utc::now(),
            execution_collector: ExecutionMetricsCollector::new(),
            token_collector: TokenMetricsCollector::new(),
            retrieval_collector: RetrievalMetricsCollector::new(),
            agent_collector: AgentMetricsCollector::new(),
            validation_collector: ValidationMetricsCollector::new(),
        };

        let report = generator.generate_report(input);

        assert_eq!(report.project_name, "test-project");
        assert_eq!(report.project_version, "1.0.0");
        assert_eq!(report.executive_summary.total_tasks_executed, 0);
    }

    #[test]
    fn test_export_to_json() {
        let generator = AuditReportGenerator::new();
        let input = AuditReportInput {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            period_start: Utc::now() - chrono::Duration::days(7),
            period_end: Utc::now(),
            execution_collector: ExecutionMetricsCollector::new(),
            token_collector: TokenMetricsCollector::new(),
            retrieval_collector: RetrievalMetricsCollector::new(),
            agent_collector: AgentMetricsCollector::new(),
            validation_collector: ValidationMetricsCollector::new(),
        };

        let report = generator.generate_report(input);

        let json = generator.export_to_json(&report).unwrap();
        assert!(json.contains("test-project"));
    }

    #[test]
    fn test_export_to_markdown() {
        let generator = AuditReportGenerator::new();
        let input = AuditReportInput {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            period_start: Utc::now() - chrono::Duration::days(7),
            period_end: Utc::now(),
            execution_collector: ExecutionMetricsCollector::new(),
            token_collector: TokenMetricsCollector::new(),
            retrieval_collector: RetrievalMetricsCollector::new(),
            agent_collector: AgentMetricsCollector::new(),
            validation_collector: ValidationMetricsCollector::new(),
        };

        let report = generator.generate_report(input);

        let markdown = generator.export_to_markdown(&report);
        assert!(markdown.contains("# Audit Report: test-project"));
        assert!(markdown.contains("## Executive Summary"));
    }
}
