package utils

// import (
// 		"github.com/docker/docker/api/types/container"
// 	"github.com/docker/docker/client"
// )
// This file contains utility functions for Docker operations, specifically building images, pulling images and pushing images.

// apiClient, err := client.NewClientWithOpts(client.FromEnv)
// if err != nil {
// 	panic(err)
// }
// defer apiClient.Close()

// ctx := context.Background()
// info, err := apiClient.Info(ctx)
// serialized_info, _ := json.MarshalIndent(info, "", "  ")
// fmt.Printf("%s\n", string(serialized_info))

// containers, err := apiClient.ContainerList(context.Background(), container.ListOptions{All: true})
// if err != nil {
// 	panic(err)
// }

// for _, ctr := range containers {
// 	fmt.Printf("%s %s (status: %s)\n", ctr.ID, ctr.Image, ctr.Status)
// }
// return
