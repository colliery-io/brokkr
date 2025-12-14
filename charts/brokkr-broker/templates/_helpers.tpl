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
{{- printf "%s-postgresql" .Release.Name }}
{{- else }}
{{- .Values.postgresql.external.host }}
{{- end }}
{{- end }}

{{/*
Database username
*/}}
{{- define "brokkr-broker.databaseUsername" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.username }}
{{- else }}
{{- .Values.postgresql.external.username }}
{{- end }}
{{- end }}

{{/*
Database password
*/}}
{{- define "brokkr-broker.databasePassword" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.password }}
{{- else }}
{{- .Values.postgresql.external.password }}
{{- end }}
{{- end }}

{{/*
Database name
*/}}
{{- define "brokkr-broker.databaseName" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.postgresql.external.database }}
{{- end }}
{{- end }}

{{/*
Database port
*/}}
{{- define "brokkr-broker.databasePort" -}}
{{- if .Values.postgresql.enabled -}}
5432
{{- else -}}
{{- .Values.postgresql.external.port -}}
{{- end -}}
{{- end }}

{{/*
TLS secret name
Returns the name of the TLS secret to use based on configuration:
- existingSecret if provided
- Generated secret name if inline certs provided
- Empty string if TLS disabled
*/}}
{{- define "brokkr-broker.tlsSecretName" -}}
{{- if .Values.tls.enabled }}
{{- if .Values.tls.existingSecret }}
{{- .Values.tls.existingSecret }}
{{- else if or .Values.tls.cert .Values.tls.key }}
{{- printf "%s-tls" (include "brokkr-broker.fullname" .) }}
{{- else if .Values.tls.certManager.enabled }}
{{- printf "%s-tls" (include "brokkr-broker.fullname" .) }}
{{- end }}
{{- end }}
{{- end }}
