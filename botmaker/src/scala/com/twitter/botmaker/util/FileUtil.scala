package com.twitter.botmaker.util

import java.io.File
import java.io.FileOutputStream
import java.io.FileWriter
import java.nio.charset.StandardCharsets
import com.google.common.io.{Files => GoogleFiles}

object FileUtil {

  def exists(file: String): Boolean = new File(file).exists()

  def delete(dir: String): Boolean = new File(dir).delete

  def writeAllBytes(file: String, bytes: Array[Byte]): Unit = {
    val fs = new FileOutputStream(file)
    fs.write(bytes)
    fs.close()
  }

  def writeAllText(file: String, text: String): Unit = {
    val writer = new FileWriter(file)
    writer.write(text)
    writer.close()
  }

  def writeAllLines(file: String, text: Seq[String]): Unit = {
    val fs = new FileWriter(file)
    val eol = System.getProperty("line.separator")
    text.foreach(line => {
      fs.write(line)
      fs.write(eol)
    })
    fs.close()
  }

  def readAllText(file: String): String = {
    GoogleFiles.toString(new File(file), StandardCharsets.UTF_8)
  }

  def readAllLines(file: String): Seq[String] = {
    scala.io.Source.fromFile(file).getLines().toList
  }

  def readAllBytes(file: String): Array[Byte] = {
    GoogleFiles.toByteArray(new File(file))
  }

  def createDirectory(dir: String): Boolean = {
    new File(dir).mkdirs
  }

}
