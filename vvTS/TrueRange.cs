using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200005F RID: 95
	[HandlerCategory("vvIndicators"), HandlerName("TrueRange")]
	public class TrueRange : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000360 RID: 864 RVA: 0x00013475 File Offset: 0x00011675
		public IList<double> Execute(ISecurity src)
		{
			return TrueRange.GenTrueRange(src.get_Bars());
		}

		// Token: 0x0600035F RID: 863 RVA: 0x00013440 File Offset: 0x00011640
		public static IList<double> GenTrueRange(IList<Bar> candles)
		{
			double[] array = new double[candles.Count];
			for (int i = 0; i < candles.Count; i++)
			{
				array[i] = TrueRange.iTrueRange(candles, i);
			}
			return array;
		}

		// Token: 0x06000361 RID: 865 RVA: 0x00013484 File Offset: 0x00011684
		public static double iTrueRange(IList<Bar> candles, int curbar)
		{
			Bar bar = candles[curbar];
			double high = bar.get_High();
			double low = bar.get_Low();
			double num = Math.Abs(high - low);
			if (curbar > 0)
			{
				double close = candles[curbar - 1].get_Close();
				num = Math.Max(num, Math.Abs(close - high));
				num = Math.Max(num, Math.Abs(close - low));
			}
			return num;
		}
	}
}
