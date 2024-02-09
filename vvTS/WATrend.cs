using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200006E RID: 110
	[HandlerCategory("vvIndicators"), HandlerName("WATrend")]
	public class WATrend : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060003E2 RID: 994 RVA: 0x00015333 File Offset: 0x00013533
		public IList<double> Execute(IList<double> src)
		{
			return WATrend.GenWATrend(src, this.Context, this.Set);
		}

		// Token: 0x060003E1 RID: 993 RVA: 0x00015244 File Offset: 0x00013444
		public static IList<double> GenWATrend(IList<double> src, IContext ctx, int _Set)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> list;
			if (_Set == 1)
			{
				list = MACD.GenMACD(src, 20, 40, 9, 0);
			}
			else
			{
				list = MACD.GenMACD(src, 12, 26, 9, 0);
			}
			IList<double> list2 = BBands.GenBBands(src, ctx, 20, 2.0, 1, 0);
			IList<double> list3 = BBands.GenBBands(src, ctx, 20, 2.0, 2, 0);
			for (int i = 1; i < count; i++)
			{
				double num = list[i] - list[i - 1];
				double num2 = list2[i] - list3[i];
				array[i] = 0.0;
				double num3 = num * num2 / 1000.0;
				if (num3 >= 0.0)
				{
					array[i] = num3;
				}
				if (num3 < 0.0)
				{
					array[i] = num3;
				}
			}
			return array;
		}

		// Token: 0x1700014F RID: 335
		public IContext Context
		{
			// Token: 0x060003E3 RID: 995 RVA: 0x00015347 File Offset: 0x00013547
			get;
			// Token: 0x060003E4 RID: 996 RVA: 0x0001534F File Offset: 0x0001354F
			set;
		}

		// Token: 0x1700014E RID: 334
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int Set
		{
			// Token: 0x060003DF RID: 991 RVA: 0x00015232 File Offset: 0x00013432
			get;
			// Token: 0x060003E0 RID: 992 RVA: 0x0001523A File Offset: 0x0001343A
			set;
		}
	}
}
