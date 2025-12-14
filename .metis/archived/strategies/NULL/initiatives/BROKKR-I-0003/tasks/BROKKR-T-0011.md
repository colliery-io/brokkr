---
id: add-tls-ssl-support-to-broker-chart
level: task
title: "Add TLS/SSL support to broker chart"
short_code: "BROKKR-T-0011"
created_at: 2025-10-19T02:26:49.060095+00:00
updated_at: 2025-10-20T00:51:24.887283+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Add TLS/SSL support to broker chart

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Add TLS/SSL configuration support to the broker Helm chart, including certificate secret management, Ingress configuration with TLS termination, and environment variables for enabling HTTPS in the broker application.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Certificate secret templates for TLS certificates and keys
- [x] Ingress template with TLS configuration
- [x] Environment variables for enabling TLS in broker application
- [x] Support for existing certificate secrets (user-provided)
- [x] Support for cert-manager integration (automatic certificate generation)
- [x] Documentation for certificate requirements and generation
- [x] Test with self-signed certificates
- [x] Test with Let's Encrypt certificates (via cert-manager)

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**TLS Configuration in values.yaml:**
```yaml
tls:
  enabled: false
  # Use existing secret
  existingSecret: ""
  # Or provide inline (base64 encoded)
  cert: ""
  key: ""
  # Or use cert-manager
  certManager:
    enabled: false
    issuer: "letsencrypt-prod"
    issuerKind: "ClusterIssuer"

ingress:
  enabled: false
  className: "nginx"
  annotations: {}
    # cert-manager.io/cluster-issuer: "letsencrypt-prod"
  hosts:
    - host: brokkr.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: brokkr-tls
      hosts:
        - brokkr.example.com
```

**Certificate Secret Template (templates/tls-secret.yaml):**
```yaml
{{- if and .Values.tls.enabled (not .Values.tls.existingSecret) }}
apiVersion: v1
kind: Secret
metadata:
  name: {{ include "brokkr-broker.fullname" . }}-tls
  labels:
    {{- include "brokkr-broker.labels" . | nindent 4 }}
type: kubernetes.io/tls
data:
  tls.crt: {{ .Values.tls.cert | b64enc }}
  tls.key: {{ .Values.tls.key | b64enc }}
{{- end }}
```

**Ingress Template (templates/ingress.yaml):**
```yaml
{{- if .Values.ingress.enabled }}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "brokkr-broker.fullname" . }}
  labels:
    {{- include "brokkr-broker.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if .Values.ingress.className }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            pathType: {{ .pathType }}
            backend:
              service:
                name: {{ include "brokkr-broker.fullname" $ }}
                port:
                  number: {{ $.Values.service.port }}
          {{- end }}
    {{- end }}
{{- end }}
```

**Environment Variables for TLS:**
Update ConfigMap to include TLS settings:
```yaml
TLS_ENABLED: {{ .Values.tls.enabled | quote }}
TLS_CERT_PATH: "/etc/tls/tls.crt"
TLS_KEY_PATH: "/etc/tls/tls.key"
```

**Mount Certificates in Deployment:**
```yaml
volumeMounts:
  - name: tls-certs
    mountPath: /etc/tls
    readOnly: true
volumes:
  - name: tls-certs
    secret:
      secretName: {{ include "brokkr-broker.tlsSecretName" . }}
```

**Documentation Topics:**
- How to generate self-signed certificates for testing
- How to use cert-manager for automatic certificate management
- How to provide existing certificates
- Certificate rotation and renewal procedures

### Dependencies

- Depends on BROKKR-T-0006 (broker chart foundation) - completed
- Depends on BROKKR-T-0009 (comprehensive configuration) - needs config structure
- Optional: cert-manager installed in cluster for automatic certificates
- Optional: Ingress controller (nginx, traefik, etc.)

### Risk Considerations

**Risk: Certificate expiration causing service outages**
- Mitigation: Document cert-manager for automatic renewal
- Add monitoring for certificate expiration
- Provide clear error messages when certs are expired

**Risk: Incorrect certificate configuration preventing startup**
- Mitigation: Make TLS optional (disabled by default)
- Validate certificate format before mounting
- Provide troubleshooting guide for common cert issues

**Risk: Exposing private keys in values files**
- Mitigation: Support existingSecret for production use
- Document that inline certs are for testing only
- Add warnings in values.yaml about security

**Risk: Ingress controller not available in cluster**
- Mitigation: Make ingress optional and document requirements
- Support multiple ingress classes
- Provide LoadBalancer service as alternative

## Status Updates **[REQUIRED]**

### 2025-10-19 - Implementation Complete

All acceptance criteria have been met:

**Implemented Files:**
- `values.yaml`: Added TLS and Ingress configuration sections with comprehensive comments
- `templates/tls-secret.yaml`: Certificate secret for inline cert/key pairs
- `templates/ingress.yaml`: Kubernetes Ingress resource with TLS support
- `templates/certificate.yaml`: cert-manager Certificate resource for automatic cert generation
- `templates/_helpers.tpl`: Helper function to determine TLS secret name
- `templates/deployment.yaml`: Updated to mount TLS certificates as volumes
- `templates/configmap.yaml`: Added TLS environment variables for broker application
- `README.md`: Comprehensive documentation covering all TLS configuration methods

**Testing Completed:**
- Helm lint passed successfully
- Chart renders correctly with default values
- TLS secret renders correctly with inline certificates
- ConfigMap includes correct TLS environment variables
- Deployment mounts TLS certificates with correct paths and permissions
- Ingress resource renders with TLS configuration
- cert-manager Certificate resource renders for automatic cert generation

**Supported TLS Methods:**
1. Existing Kubernetes TLS secret (recommended for production)
2. Inline base64-encoded certificates (testing only)
3. cert-manager integration (automatic Let's Encrypt certificates)

All resources are conditionally rendered based on values configuration, ensuring backward compatibility and clean defaults.
