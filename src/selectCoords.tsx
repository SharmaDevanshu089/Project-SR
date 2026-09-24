import React, { useState } from "react";
import mapboxgl from "mapbox-gl";
import "mapbox-gl/dist/mapbox-gl.css";

mapboxgl.accessToken = import.meta.env.VITE_MAPBOX_TOKEN;



const CoordinatePicker: React.FC = () => {
    const [coords, setCoords] = useState<{ lat: number; lng: number } | null>(null);

    React.useEffect(() => {
        const map = new mapboxgl.Map({
            container: "map",
            style: "mapbox://styles/mapbox/streets-v12",
            center: [0, 20],
            zoom: 2,
        });

        map.on("click", (e) => {
            const lng = parseFloat(e.lngLat.lng.toFixed(6));
            const lat = parseFloat(e.lngLat.lat.toFixed(6));
            setCoords({ lat, lng });
            new mapboxgl.Marker().setLngLat([lng, lat]).addTo(map);
        });

        return () => map.remove();
    }, []);

    const handleSend = () => {
        if (coords) {
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
                <form>
                    <button
                        onClick={handleSend}
                        disabled={!coords}
                        style={{
                            padding: "10px 20px",
                            background: "#0078d7",
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
