package main

import (
	"log"
	"os/exec"
)

func main() {
	cmd := exec.Command("git", "diff", "--staged")
	diff, err := cmd.Output()
	log.Println(string(diff))
	if err != nil {
		log.Fatal(err)
	}
}
