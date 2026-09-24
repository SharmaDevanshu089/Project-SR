import React, { createContext, useContext, useState, ReactNode } from "react";

export interface Coordinates {
    lat: number;
    lng: number;
}

interface CoordinateContextType {
    coords: Coordinates | null;
    setCoords: (coords: Coordinates | null) => void;
}

const CoordinateContext = createContext<CoordinateContextType | undefined>(undefined);

export const CoordinateProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [coords, setCoords] = useState<Coordinates | null>(null);

    return (
        <CoordinateContext.Provider value={{ coords, setCoords }}>
            {children}
        </CoordinateContext.Provider>
    );
};

export const useCoordinates = (): CoordinateContextType => {
    const context = useContext(CoordinateContext);
    if (!context) {
        throw new Error("useCoordinates must be used within a CoordinateProvider");
    }
    return context;
};
