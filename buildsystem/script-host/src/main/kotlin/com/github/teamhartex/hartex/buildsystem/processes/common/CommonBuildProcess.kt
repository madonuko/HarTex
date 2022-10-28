package com.github.teamhartex.hartex.buildsystem.processes.common

import com.github.teamhartex.hartex.buildsystem.CargoBuildProfile
import com.github.teamhartex.hartex.buildsystem.Project
import com.github.teamhartex.hartex.buildsystem.ProjectBuildTool
import com.github.teamhartex.hartex.buildsystem.ProjectType
import java.io.File

class CommonBuildProcess {
  companion object {
    fun new(projectToBuild: Project, args: List<String>): Process {
      val processBuilder = ProcessBuilder()
        .redirectOutput(ProcessBuilder.Redirect.PIPE)

      when (projectToBuild.projectType to projectToBuild.buildTool) {
        ProjectType.RUST to ProjectBuildTool.CARGO -> {
          processBuilder.command("cargo", "build")

          when (projectToBuild.cargoBuildProfile) {
            CargoBuildProfile.RELEASE -> {
              processBuilder.command().add("--release")
            }
            else -> {}
          }
        }
        ProjectType.TYPESCRIPT to ProjectBuildTool.YARN -> processBuilder.command("yarn", "build")
      }

      val process = processBuilder.directory(File(System.getProperty("user.dir") + """/${args[2]}"""))
        .start()
      val outputReader = process.errorStream.bufferedReader()
      var line = outputReader.readLine()
      while (line != null) {
        println(line)
        line = outputReader.readLine()
      }

      return process
    }
  }
}