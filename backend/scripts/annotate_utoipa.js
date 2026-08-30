const fs = require('fs');

// file -> [ [fnName, attribute], ... ]  (attribute paths include the nest prefixes)
const TABLE = {
  'handlers/auth.rs': [
    ['google_login_url', `#[utoipa::path(post, path = "/api/v1/auth/google", tag = "auth", operation_id = "auth_google_login_url", responses((status = 200, body = GoogleLoginUrlResponse)))]`],
    ['google_login', `#[utoipa::path(get, path = "/api/v1/auth/google/login", tag = "auth", operation_id = "auth_google_login", responses((status = 302, description = "Redirect to Google consent")))]`],
    ['google_callback', `#[utoipa::path(get, path = "/api/v1/auth/google/callback", tag = "auth", operation_id = "auth_google_callback", responses((status = 302, description = "Redirect to frontend with session cookies")))]`],
    ['refresh', `#[utoipa::path(post, path = "/api/v1/auth/refresh", tag = "auth", operation_id = "auth_refresh", responses((status = 204)))]`],
    ['logout', `#[utoipa::path(post, path = "/api/v1/auth/logout", tag = "auth", operation_id = "auth_logout", responses((status = 204)))]`],
    ['me', `#[utoipa::path(get, path = "/api/v1/auth/me", tag = "auth", operation_id = "auth_me", responses((status = 200, body = AuthMeResponse)))]`],
  ],
  'handlers/users.rs': [
    ['onboarding', `#[utoipa::path(post, path = "/api/v1/users/onboarding", tag = "users", operation_id = "users_onboarding", request_body = OnboardingRequest, responses((status = 200, body = UserResponse)))]`],
    ['get_me', `#[utoipa::path(get, path = "/api/v1/users/me", tag = "users", operation_id = "users_get_me", responses((status = 200, body = UserResponse)))]`],
    ['update_me', `#[utoipa::path(patch, path = "/api/v1/users/me", tag = "users", operation_id = "users_update_me", request_body = UpdateProfileRequest, responses((status = 200, body = UserResponse)))]`],
  ],
  'handlers/vehicles.rs': [
    ['create', `#[utoipa::path(post, path = "/api/v1/vehicles", tag = "vehicles", operation_id = "vehicles_create", request_body = VehicleCreateRequest, responses((status = 200, body = VehicleResponse)))]`],
    ['list', `#[utoipa::path(get, path = "/api/v1/vehicles", tag = "vehicles", operation_id = "vehicles_list", responses((status = 200, body = [VehicleResponse])))]`],
    ['get_one', `#[utoipa::path(get, path = "/api/v1/vehicles/{vehicle_id}", tag = "vehicles", operation_id = "vehicles_get", params(("vehicle_id" = Uuid, Path, description = "Vehicle id")), responses((status = 200, body = VehicleResponse)))]`],
    ['update', `#[utoipa::path(patch, path = "/api/v1/vehicles/{vehicle_id}", tag = "vehicles", operation_id = "vehicles_update", params(("vehicle_id" = Uuid, Path)), request_body = VehicleUpdateRequest, responses((status = 200, body = VehicleResponse)))]`],
    ['delete', `#[utoipa::path(delete, path = "/api/v1/vehicles/{vehicle_id}", tag = "vehicles", operation_id = "vehicles_delete", params(("vehicle_id" = Uuid, Path)), responses((status = 204)))]`],
  ],
  'handlers/breakdowns.rs': [
    ['create', `#[utoipa::path(post, path = "/api/v1/breakdowns", tag = "breakdowns", operation_id = "breakdowns_create", request_body = BreakdownCreateRequest, responses((status = 202, body = JobResponse)))]`],
    ['get_one', `#[utoipa::path(get, path = "/api/v1/breakdowns/{id}", tag = "breakdowns", operation_id = "breakdowns_get", params(("id" = Uuid, Path)), responses((status = 200, body = BreakdownDetail)))]`],
    ['cancel', `#[utoipa::path(post, path = "/api/v1/breakdowns/{id}/cancel", tag = "breakdowns", operation_id = "breakdowns_cancel", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['upload_media', `#[utoipa::path(post, path = "/api/v1/breakdowns/{id}/media", tag = "breakdowns", operation_id = "breakdowns_upload_media", params(("id" = Uuid, Path)), request_body(content = "multipart/form-data"), responses((status = 200, body = [BreakdownMediaResponse])))]`],
    ['download_media', `#[utoipa::path(get, path = "/api/v1/breakdowns/{id}/media/{media_id}/content", tag = "breakdowns", operation_id = "breakdowns_download_media", params(("id" = Uuid, Path), ("media_id" = Uuid, Path)), responses((status = 200, description = "Stored bytes")))]`],
    ['diagnosis', `#[utoipa::path(get, path = "/api/v1/breakdowns/{id}/diagnosis", tag = "breakdowns", operation_id = "breakdowns_diagnosis", params(("id" = Uuid, Path)), responses((status = 200, body = DiagnosisResponse)))]`],
  ],
  'handlers/mechanics.rs': [
    ['get_me', `#[utoipa::path(get, path = "/api/v1/mechanics/me", tag = "mechanics", operation_id = "mechanics_get_me", responses((status = 200, body = MechanicProfileResponse)))]`],
    ['onboarding', `#[utoipa::path(post, path = "/api/v1/mechanics/onboarding", tag = "mechanics", operation_id = "mechanics_onboarding", request_body = MechanicOnboardingRequest, responses((status = 200, body = MechanicProfileResponse)))]`],
    ['update_me', `#[utoipa::path(patch, path = "/api/v1/mechanics/me", tag = "mechanics", operation_id = "mechanics_update_me", request_body = MechanicProfileUpdateRequest, responses((status = 200, body = MechanicProfileResponse)))]`],
    ['set_availability', `#[utoipa::path(post, path = "/api/v1/mechanics/availability", tag = "mechanics", operation_id = "mechanics_set_availability", request_body = AvailabilityRequest, responses((status = 200, body = MechanicProfileResponse)))]`],
    ['push_location', `#[utoipa::path(post, path = "/api/v1/mechanics/location", tag = "mechanics", operation_id = "mechanics_push_location", request_body = MechanicLocationPing, responses((status = 200)))]`],
    ['requests', `#[utoipa::path(get, path = "/api/v1/mechanics/requests", tag = "mechanics", operation_id = "mechanics_requests", params(("latitude" = f64, Query, description = "Current latitude"), ("longitude" = f64, Query, description = "Current longitude")), responses((status = 200, body = [MechanicFeedItem])))]`],
  ],
  'handlers/jobs.rs': [
    ['list_mine', `#[utoipa::path(get, path = "/api/v1/jobs", tag = "jobs", operation_id = "jobs_list_mine", responses((status = 200, body = [JobResponse])))]`],
    ['get_job', `#[utoipa::path(get, path = "/api/v1/jobs/{id}", tag = "jobs", operation_id = "jobs_get", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['status_history', `#[utoipa::path(get, path = "/api/v1/jobs/{id}/status-history", tag = "jobs", operation_id = "jobs_status_history", params(("id" = Uuid, Path)), responses((status = 200, body = [JobStatusHistoryEntry])))]`],
    ['mark_arrived', `#[utoipa::path(post, path = "/api/v1/jobs/{id}/arrived", tag = "jobs", operation_id = "jobs_mark_arrived", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['start_repair', `#[utoipa::path(post, path = "/api/v1/jobs/{id}/start-repair", tag = "jobs", operation_id = "jobs_start_repair", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['complete', `#[utoipa::path(post, path = "/api/v1/jobs/{id}/complete", tag = "jobs", operation_id = "jobs_complete", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['start_travel', `#[utoipa::path(post, path = "/api/v1/jobs/{id}/start-travel", tag = "jobs", operation_id = "jobs_start_travel", params(("id" = Uuid, Path)), responses((status = 200, body = JobResponse)))]`],
    ['list_mine_dupe_placeholder', ''],
    ['push_location', `#[utoipa::path(post, path = "/api/v1/jobs/{id}/location", tag = "jobs", operation_id = "jobs_push_location", params(("id" = Uuid, Path)), request_body = MechanicLocationPing, responses((status = 200)))]`],
  ],
  'handlers/offers.rs': [
    ['create_offer', `#[utoipa::path(post, path = "/api/v1/jobs/{job_id}/offers", tag = "offers", operation_id = "offers_create", params(("job_id" = Uuid, Path)), request_body = OfferCreateRequest, responses((status = 200, body = OfferResponse)))]`],
    ['list_offers', `#[utoipa::path(get, path = "/api/v1/jobs/{job_id}/offers", tag = "offers", operation_id = "offers_list", params(("job_id" = Uuid, Path)), responses((status = 200, body = [OfferResponse])))]`],
    ['select_offer', `#[utoipa::path(post, path = "/api/v1/offers/{offer_id}/select", tag = "offers", operation_id = "offers_select", params(("offer_id" = Uuid, Path)), responses((status = 200, body = OfferSelectionResponse)))]`],
    ['withdraw_offer', `#[utoipa::path(post, path = "/api/v1/offers/{offer_id}/withdraw", tag = "offers", operation_id = "offers_withdraw", params(("offer_id" = Uuid, Path)), responses((status = 200)))]`],
  ],
  'handlers/payments.rs': [
    ['pay', `#[utoipa::path(post, path = "/api/v1/jobs/{job_id}/pay", tag = "payments", operation_id = "payments_pay", params(("job_id" = Uuid, Path)), request_body = PaymentInitiateRequest, responses((status = 200, body = PaymentResponse)))]`],
  ],
  'handlers/ratings.rs': [
    ['create_rating', `#[utoipa::path(post, path = "/api/v1/ratings", tag = "ratings", operation_id = "ratings_create", request_body = RatingCreateRequest, responses((status = 200, body = RatingResponse)))]`],
    ['mechanic_reviews', `#[utoipa::path(get, path = "/api/v1/ratings/mechanics/{mechanic_id}/reviews", tag = "ratings", operation_id = "ratings_mechanic_reviews", params(("mechanic_id" = Uuid, Path)), responses((status = 200, body = [RatingResponse])))]`],
  ],
  'handlers/notifications.rs': [
    ['list_notifications', `#[utoipa::path(get, path = "/api/v1/notifications", tag = "notifications", operation_id = "notifications_list", responses((status = 200, body = [NotificationView])))]`],
  ],
  'handlers/payments_history.rs': [
    ['my_payments', `#[utoipa::path(get, path = "/api/v1/payments", tag = "payments", operation_id = "payments_my", params(("limit" = i64, Query, description = "Max rows")), responses((status = 200, body = [PaymentView])))]`],
    ['my_earnings', `#[utoipa::path(get, path = "/api/v1/mechanics/payments", tag = "payments", operation_id = "payments_mechanic_earnings", responses((status = 200, body = [MechanicPaymentView])))]`],
  ],
  'handlers/agent.rs': [
    ['diagnose', `#[utoipa::path(post, path = "/api/v1/agent/diagnose", tag = "agent", operation_id = "agent_diagnose", request_body = AgentDiagnoseRequest, responses((status = 200, body = AgentDiagnosisDto)))]`],
  ],
  'handlers/admin.rs': [
    ['login', `#[utoipa::path(post, path = "/api/v1/admin/login", tag = "admin", operation_id = "admin_login", request_body = AdminLoginRequest, responses((status = 200, body = LoggedInResponse)))]`],
    ['overview', `#[utoipa::path(get, path = "/api/v1/admin/overview", tag = "admin", operation_id = "admin_overview", responses((status = 200, body = AdminOverviewView)))]`],
    ['users', `#[utoipa::path(get, path = "/api/v1/admin/users", tag = "admin", operation_id = "admin_users", params(("role" = String, Query, description = "Filter by role"), ("limit" = i64, Query, description = "Max rows")), responses((status = 200, body = [AdminUserView])))]`],
    ['set_user_status', `#[utoipa::path(post, path = "/api/v1/admin/users/{user_id}/status", tag = "admin", operation_id = "admin_set_user_status", params(("user_id" = Uuid, Path)), request_body = SetStatusRequest, responses((status = 204)))]`],
    ['mechanics', `#[utoipa::path(get, path = "/api/v1/admin/mechanics", tag = "admin", operation_id = "admin_mechanics", responses((status = 200, body = [MechanicProfileResponse])))]`],
    ['verify_mechanic', `#[utoipa::path(post, path = "/api/v1/admin/mechanics/{mechanic_id}/verify", tag = "admin", operation_id = "admin_verify_mechanic", params(("mechanic_id" = Uuid, Path)), request_body = VerifyRequest, responses((status = 204)))]`],
    ['jobs', `#[utoipa::path(get, path = "/api/v1/admin/jobs", tag = "admin", operation_id = "admin_jobs", params(("limit" = i64, Query)), responses((status = 200, body = [AdminJobView])))]`],
    ['payments', `#[utoipa::path(get, path = "/api/v1/admin/payments", tag = "admin", operation_id = "admin_payments", params(("limit" = i64, Query)), responses((status = 200, body = [AdminPaymentView])))]`],
    ['agent_runs', `#[utoipa::path(get, path = "/api/v1/admin/agent-runs", tag = "admin", operation_id = "admin_agent_runs", params(("limit" = i64, Query)), responses((status = 200, body = [AgentRunView])))]`],
  ],
};

let inserted = 0;
for (const [file, entries] of Object.entries(TABLE)) {
  let src = fs.readFileSync('crates/toolfix-api/src/' + file, 'utf8');
  for (const [fnName, attr] of entries) {
    if (!attr) continue;
    const needle = `pub async fn ${fnName}(`;
    if (!src.includes(needle)) {
      console.error('NOT FOUND:', file, fnName);
      continue;
    }
    src = src.replace(needle, attr + '\npub async fn ' + fnName + '(');
    inserted++;
  }
  fs.writeFileSync('crates/toolfix-api/src/' + file, src);
}
console.log('attributes inserted:', inserted);
