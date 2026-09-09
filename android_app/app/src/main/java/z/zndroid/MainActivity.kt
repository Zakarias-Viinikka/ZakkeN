package z.zndroid

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import z.zndroid.ui.theme.ZndroidTheme
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import z.zndroid.MainPages.NavPage
import z.zndroid.MainPages.ViewPage.ViewPage
import z.zndroid.components.MainContainer
import androidx.compose.runtime.LaunchedEffect
import z.zndroid.Storage.SessionManager

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            ZndroidTheme {
                MainContainer {
                    val navController = rememberNavController()
                    
                    LaunchedEffect(navController) {
                        navController.addOnDestinationChangedListener { _, _, _ ->
                            SessionManager.incrementAndStore()
                        }
                    }

                    NavHost(
                        navController = navController,
                        startDestination = "nav_page",
                        enterTransition = { EnterTransition.None },
                        exitTransition = { ExitTransition.None },
                        popEnterTransition = { EnterTransition.None },
                        popExitTransition = { ExitTransition.None }
                    ) {
                        composable("nav_page") {
                            NavPage(
                                onOpenPage = { title ->
                                    navController.navigate("view_page/$title")
                                },
                                onOpenAdminView = { title ->
                                    navController.navigate("admin_view/$title")
                                },
                                onOpenDbInspector = {
                                    navController.navigate("db_gui")
                                },
                                onOpenTests = {
                                    navController.navigate("test_results")
                                }
                            )
                        }
                        composable("test_results") {
                            z.zndroid.Tests.TestResultsPage(onBack = { navController.popBackStack() })
                        }
                        composable(
                            route = "view_page/{pageId}",
                            arguments = listOf(navArgument("pageId") { type = NavType.StringType })
                        ) { backStackEntry ->
                            val id = backStackEntry.arguments?.getString("pageId") ?: ""
                            ViewPage(
                                pageId = id,
                                onBack = { navController.popBackStack() },
                                onOpenAdminView = { navController.navigate("admin_view/$id") }
                            )
                        }
                        composable(
                            route = "admin_view/{pageId}",
                            arguments = listOf(navArgument("pageId") { type = NavType.StringType })
                        ) { backStackEntry ->
                            val id = backStackEntry.arguments?.getString("pageId") ?: ""
                            z.zndroid.MainPages.ViewPage.AdminViewOfPage(
                                pageId = id,
                                onBack = { navController.popBackStack() }
                            )
                        }
                        composable("db_gui") {
                            DbGui(
                                onInspectTable = { tableName ->
                                    navController.navigate("inspect_table/$tableName")
                                },
                                onViewSchema = {
                                    navController.navigate("schema_viewer")
                                }
                            )
                        }
                        composable("schema_viewer") {
                            SchemaGui(onBack = { navController.popBackStack() })
                        }
                        composable(
                            route = "inspect_table/{tableName}",
                            arguments = listOf(navArgument("tableName") { type = NavType.StringType })
                        ) { backStackEntry ->
                            val tableName = backStackEntry.arguments?.getString("tableName") ?: ""
                            InspectTable(
                                tableName = tableName,
                                onBack = { navController.popBackStack() }
                            )
                        }
                    }
                }
            }
        }
    }
}
