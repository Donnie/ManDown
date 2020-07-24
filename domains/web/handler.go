package web

import (
	"net/http"
	"net/url"
	"strings"
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

func Sanitise(site string) string {
	web, err := url.Parse(strings.ToLower(site))
	if err != nil {
		return ""
	}

	if web.Scheme != "http" {
		web.Scheme = "https"
	}

	web, _ = url.Parse(web.String())
	return web.Scheme + "://" + web.Hostname()
}
