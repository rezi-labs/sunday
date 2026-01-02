# Private configuration for Sunday application
# This file contains secrets and should not be committed to version control

# Database configuration for local development
export pg_admin := "postgres"
export pg_pw := "postgres"

# Environment variables for the application
export DATABASE_URL := "postgres://postgres:postgres@localhost:5432/sunday?sslmode=disable"
export SESSION_SECRET_KEY := "EHICtXDIS6FmOw5hiwvnJICz+3/Sf9WmFHLnCshbuE4cLDomQVXH1/+FKuATnpxINyLWgdpEipoQmFadj+CsNQ=="
export PORT := "3000"
export HOST := "localhost"
export RUST_LOG := "info"