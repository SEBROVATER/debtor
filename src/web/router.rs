#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteMethod {
    Get,
    Post,
    Patch,
    Delete,
}

impl RouteMethod {
    pub fn is_state_changing(self) -> bool {
        matches!(
            self,
            RouteMethod::Post | RouteMethod::Patch | RouteMethod::Delete
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSpec {
    pub method: RouteMethod,
    pub path: &'static str,
    pub requires_auth: bool,
    pub csrf_protected: bool,
    pub handler: &'static str,
}

pub fn route_specs() -> Vec<RouteSpec> {
    vec![
        RouteSpec {
            method: RouteMethod::Get,
            path: "/health",
            requires_auth: false,
            csrf_protected: false,
            handler: "health_handler",
        },
        RouteSpec {
            method: RouteMethod::Get,
            path: "/login",
            requires_auth: false,
            csrf_protected: false,
            handler: "auth_login_page",
        },
        RouteSpec {
            method: RouteMethod::Post,
            path: "/login",
            requires_auth: false,
            csrf_protected: true,
            handler: "auth_login_submit",
        },
        RouteSpec {
            method: RouteMethod::Post,
            path: "/logout",
            requires_auth: true,
            csrf_protected: true,
            handler: "auth_logout_submit",
        },
        RouteSpec {
            method: RouteMethod::Get,
            path: "/dashboard",
            requires_auth: true,
            csrf_protected: false,
            handler: "dashboard_handler",
        },
        RouteSpec {
            method: RouteMethod::Post,
            path: "/groups",
            requires_auth: true,
            csrf_protected: true,
            handler: "groups_create",
        },
        RouteSpec {
            method: RouteMethod::Get,
            path: "/groups/{group_id}",
            requires_auth: true,
            csrf_protected: false,
            handler: "groups_detail",
        },
        RouteSpec {
            method: RouteMethod::Patch,
            path: "/groups/{group_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "groups_update",
        },
        RouteSpec {
            method: RouteMethod::Delete,
            path: "/groups/{group_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "groups_delete",
        },
        RouteSpec {
            method: RouteMethod::Post,
            path: "/groups/{group_id}/members",
            requires_auth: true,
            csrf_protected: true,
            handler: "members_create",
        },
        RouteSpec {
            method: RouteMethod::Patch,
            path: "/groups/{group_id}/members/{member_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "members_update",
        },
        RouteSpec {
            method: RouteMethod::Delete,
            path: "/groups/{group_id}/members/{member_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "members_delete",
        },
        RouteSpec {
            method: RouteMethod::Post,
            path: "/groups/{group_id}/expenses",
            requires_auth: true,
            csrf_protected: true,
            handler: "expenses_create",
        },
        RouteSpec {
            method: RouteMethod::Patch,
            path: "/groups/{group_id}/expenses/{expense_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "expenses_update",
        },
        RouteSpec {
            method: RouteMethod::Delete,
            path: "/groups/{group_id}/expenses/{expense_id}",
            requires_auth: true,
            csrf_protected: true,
            handler: "expenses_delete",
        },
        RouteSpec {
            method: RouteMethod::Get,
            path: "/groups/{group_id}/debts",
            requires_auth: true,
            csrf_protected: false,
            handler: "debts_summary",
        },
    ]
}
