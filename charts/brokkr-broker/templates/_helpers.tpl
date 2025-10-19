{{/*
Expand the name of the chart.
*/}}
{{- define "brokkr-broker.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "brokkr-broker.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "brokkr-broker.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "brokkr-broker.labels" -}}
helm.sh/chart: {{ include "brokkr-broker.chart" . }}
{{ include "brokkr-broker.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "brokkr-broker.selectorLabels" -}}
app.kubernetes.io/name: {{ include "brokkr-broker.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Database host
*/}}
{{- define "brokkr-broker.databaseHost" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "brokkr-broker.fullname" .) }}
{{- else }}
{{- .Values.postgresql.host }}
{{- end }}
{{- end }}
