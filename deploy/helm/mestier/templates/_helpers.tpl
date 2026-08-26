{{/*
Base name for the release, truncated to fit Kubernetes' 63-char label limit.
*/}}
{{- define "mestier.name" -}}
{{- .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Fully qualified app name: <release>-mestier, or just the release name if it
already contains "mestier".
*/}}
{{- define "mestier.fullname" -}}
{{- if contains .Chart.Name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}

{{/*
Common labels, applied to every resource.
*/}}
{{- define "mestier.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{ include "mestier.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.global.labels }}
{{ toYaml . }}
{{- end }}
{{- end -}}

{{/*
Base selector labels, shared by both components. Component-specific templates
append their own "app.kubernetes.io/component" on top of this.
*/}}
{{- define "mestier.selectorLabels" -}}
app.kubernetes.io/name: {{ include "mestier.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Name of the ServiceAccount pods should use.
*/}}
{{- define "mestier.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{ default (include "mestier.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
{{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}

{{/*
Name of the Secret the api pod loads via envFrom: the chart-managed one
unless the caller points at an existing Secret.
*/}}
{{- define "mestier.api.secretName" -}}
{{- .Values.api.secret.existingSecret | default (printf "%s-api" (include "mestier.fullname" .)) -}}
{{- end -}}

{{/*
Resolves an image ref as "repository:tag", falling back to Chart.AppVersion.
Usage: include "mestier.image" (dict "image" .Values.api.image "chart" .Chart)
*/}}
{{- define "mestier.image" -}}
{{- printf "%s:%s" .image.repository (.image.tag | default .chart.AppVersion) -}}
{{- end -}}

{{/*
Renders one `env:` entry for a single api.secret key, resolving it the same
way templates/api/secret.yaml decides what to put in the chart-managed
Secret: a per-key valueFrom override wins, then an existingSecret (assumed
to carry every key), then the chart-managed Secret — and if none of those
apply (no override, no existingSecret, and the plain value is empty), no
entry is rendered at all. Keep this the single place that resolution lives:
deployment.yaml and the migrations Job both call it, so a key sourced two
different ways in two different templates can't drift again — see the
api.secret.valueFrom fix on why "drift" here isn't hypothetical.
Usage: include "mestier.api.secretEnvEntry" (dict "root" $ "key" "DATABASE_PASSWORD")
*/}}
{{- define "mestier.api.secretEnvEntry" -}}
{{- $root := .root -}}
{{- $key := .key -}}
{{- $override := index $root.Values.api.secret.valueFrom $key -}}
{{- $plain := index $root.Values.api.secret $key -}}
{{- if or $override $root.Values.api.secret.existingSecret $plain }}
- name: {{ $key }}
  valueFrom:
    secretKeyRef:
      {{- if $override }}
      name: {{ $override.secretName }}
      key: {{ $override.key }}
      {{- else }}
      name: {{ include "mestier.api.secretName" $root }}
      key: {{ $key }}
      {{- end }}
{{- end }}
{{- end -}}
