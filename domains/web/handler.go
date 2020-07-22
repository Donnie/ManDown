package web

import (
	"net/http"
	"net/url"
)

func CheckHealth(site string) (int, error) {
	web, err := url.ParseRequestURI(site)
	if err != nil {
		return 0, err
	}

	resp, err := http.Get(web.String())
	if err != nil {
		return 1, nil
	}

	return resp.StatusCode, nil
}
