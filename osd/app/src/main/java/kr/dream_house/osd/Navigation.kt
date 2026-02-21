package kr.dream_house.osd

import android.os.Bundle
import androidx.navigation.NavHostController

interface Route {
    fun route(): String
    fun qualifiedRoute(): String = route()
}

interface NestedNavigation {
    fun parentRoute(): String
    fun routePattern(): String = "${parentRoute()}?subroute={subroute}"
    fun Bundle?.getSubroute(): String? = this?.getString("subroute")
}

interface NestedRoute: Route {
    val navigation: NestedNavigation
    override fun qualifiedRoute(): String = "${navigation.parentRoute()}?subroute=${route()}"
}

sealed class Navigation(private val route: String): Route {
    override fun route(): String = route

    companion object {
        fun default(): Navigation = MainScreen.default()
    }

    sealed class MainScreen(route: String) : Navigation(route), NestedRoute {
        override val navigation: NestedNavigation = MainScreen

        companion object : NestedNavigation {
            override fun parentRoute(): String = "main"
            fun default(): MainScreen = Mixer
        }

        object Mixer : MainScreen("mixer")
        object UnitInformation : MainScreen("unitInformation")
    }

    object Config : Navigation("config")
}

fun NavHostController.navigate(navigation: Navigation, nested: Boolean = false) {
    if (nested) {
        if (navigation !is NestedRoute) {
            throw IllegalArgumentException("Navigation $navigation is not a nested route.")
        } else {
            navigate(route = navigation.route())
        }
    } else {
        navigate(route = navigation.qualifiedRoute())
    }
}

