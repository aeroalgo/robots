using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000146 RID: 326
	[HandlerCategory("vvMACD")]
	public class MACD : MACDBase, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A0A RID: 2570 RVA: 0x0002A206 File Offset: 0x00028406
		public IList<double> Execute(IList<double> src)
		{
			return base.CalcMACD(src, 12, 26);
		}

		// Token: 0x06000A0B RID: 2571 RVA: 0x0002A214 File Offset: 0x00028414
		public static IList<double> GenMACD(IList<double> src, int fastMaperiod, int slowMaperiod, int signalperiod, int ResultType)
		{
			IList<double> list = EMA.GenEMA(src, fastMaperiod);
			IList<double> list2 = EMA.GenEMA(src, slowMaperiod);
			double[] array = new double[list.Count];
			for (int i = 0; i < list.Count; i++)
			{
				array[i] = list[i] - list2[i];
			}
			if (ResultType == 1)
			{
				return EMA.GenEMA(array, signalperiod);
			}
			return array;
		}

		// Token: 0x06000A0C RID: 2572 RVA: 0x0002A275 File Offset: 0x00028475
		public static double iMACD(IList<double> src, IContext ctx, int fastMaperiod, int slowMaperiod, int signalperiod, int ResultType, int barNum)
		{
			return 0.0;
		}
	}
}
