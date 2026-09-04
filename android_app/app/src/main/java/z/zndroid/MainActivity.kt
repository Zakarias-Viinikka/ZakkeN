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
import z.zndroid.MainPages.ViewPage
import z.zndroid.components.MainContainer

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            ZndroidTheme {
                MainContainer {
                    val navController = rememberNavController()
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
                                onOpenDbInspector = {
                                    navController.navigate("db_gui")
                                }
                            )
                        }
                        composable(
                            route = "view_page/{pageTitle}",
                            arguments = listOf(navArgument("pageTitle") { type = NavType.StringType })
                        ) { backStackEntry ->
                            val title = backStackEntry.arguments?.getString("pageTitle") ?: ""
                            ViewPage(
                                pageTitle = title,
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
