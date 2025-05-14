using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A1 RID: 417
	[HandlerCategory("vvAverages"), HandlerName("TriMAgen")]
	public class TriMAgen : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D31 RID: 3377 RVA: 0x00039FF0 File Offset: 0x000381F0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("trimagen", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => TriMAgen.GenTriMAgen(src, this.Period));
		}

		// Token: 0x06000D2F RID: 3375 RVA: 0x00039F20 File Offset: 0x00038120
		public static IList<double> GenTriMAgen(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = TriMAgen.iTriMAgen(src, period, i);
				}
			}
			return array;
		}

		// Token: 0x06000D30 RID: 3376 RVA: 0x00039F68 File Offset: 0x00038168
		public static double iTriMAgen(IList<double> price, int period, int barNum)
		{
			int period2 = Convert.ToInt32(Math.Floor((double)(period + 1) * 0.5));
			int num = Convert.ToInt32(Math.Ceiling((double)(period + 1) * 0.5));
			double num2 = 0.0;
			for (int i = 0; i < num; i++)
			{
				num2 += SMA.iSMA(price, period2, barNum - i);
			}
			return num2 / (double)num;
		}

		// Token: 0x1700044A RID: 1098
		public IContext Context
		{
			// Token: 0x06000D32 RID: 3378 RVA: 0x0003A05C File Offset: 0x0003825C
			get;
			// Token: 0x06000D33 RID: 3379 RVA: 0x0003A064 File Offset: 0x00038264
			set;
		}

		// Token: 0x17000449 RID: 1097
		[HandlerParameter(true, "14", Min = "5", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000D2D RID: 3373 RVA: 0x00039F0D File Offset: 0x0003810D
			get;
			// Token: 0x06000D2E RID: 3374 RVA: 0x00039F15 File Offset: 0x00038115
			set;
		}
	}
}
