import React, { useRef, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import mapboxgl from "mapbox-gl";
import "mapbox-gl/dist/mapbox-gl.css";
import { useCoordinates } from "./CoordinateContext";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;

const CoordinatePicker: React.FC = () => {
    const { coords, setCoords } = useCoordinates();
    const navigate = useNavigate();
    const markerRef = useRef<mapboxgl.Marker | null>(null);

    useEffect(() => {
        const map = new mapboxgl.Map({
            container: "map",
            style: "mapbox://styles/mapbox/streets-v12",
            center: coords ? [coords.lng, coords.lat] : [0, 20],
            zoom: coords ? 9 : 2,
        });

        if (coords) {
            markerRef.current = new mapboxgl.Marker()
                .setLngLat([coords.lng, coords.lat])
                .addTo(map);
        }

        map.on("click", (e) => {
            const lng = parseFloat(e.lngLat.lng.toFixed(6));
            const lat = parseFloat(e.lngLat.lat.toFixed(6));
            setCoords({ lat, lng });

            if (markerRef.current) {
                markerRef.current.setLngLat([lng, lat]);
            } else {
                markerRef.current = new mapboxgl.Marker().setLngLat([lng, lat]).addTo(map);
            }
        });

        return () => {
            map.remove();
        };
    }, []);

    const handleSend = (e: React.MouseEvent<HTMLButtonElement>) => {
        e.preventDefault();
        if (coords) {
            navigate("/execute-fetch");
            alert(`Coordinates submitted:\nLat: ${coords.lat}, Lng: ${coords.lng}`);
        }
    };

    return (
        <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
            <div id="map" style={{ flex: 1 }} />
            <div style={{ padding: "10px", background: "#f0f0f0" }}>
                {coords ? (
                    <p>
                        Selected: Lat {coords.lat}, Lng {coords.lng}
                    </p>
                ) : (
                    <p>Click on the map to select coordinates</p>
                )}
                <form onSubmit={(e) => e.preventDefault()}>
                    <button
                        type="button"
                        onClick={handleSend}
                        disabled={!coords}
                        style={{
                            padding: "10px 20px",
                            background: coords ? "#0078d7" : "#cccccc",
                            color: "white",
                            border: "none",
                            borderRadius: "4px",
                            cursor: coords ? "pointer" : "not-allowed",
                        }}
                    >
                        Send
                    </button>
                </form>
            </div>
        </div>
    );
};

export default CoordinatePicker;

