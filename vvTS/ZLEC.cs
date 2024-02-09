using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001AA RID: 426
	[HandlerCategory("vvAverages"), HandlerName("Zerolag EC(ZEMA)")]
	public class ZLEC : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D83 RID: 3459 RVA: 0x0003B32C File Offset: 0x0003952C
		public IList<double> Execute(IList<double> src)
		{
			return ZLEC.GenZLEC(src, this.Length, this.GainLimit);
		}

		// Token: 0x06000D82 RID: 3458 RVA: 0x0003B1D0 File Offset: 0x000393D0
		public static IList<double> GenZLEC(IList<double> src, int length, int gainLimit)
		{
			double num = 2.0 / (Convert.ToDouble(length) + 1.0);
			double num2 = 0.0;
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			array[0] = src[0];
			for (int i = 1; i < src.Count; i++)
			{
				array[i] = num * src[i] + (1.0 - num) * array[i - 1];
			}
			array2[0] = src[0];
			for (int j = 1; j < src.Count; j++)
			{
				double num3 = 1000000.0;
				for (int k = -gainLimit; k <= gainLimit; k++)
				{
					double num4 = (double)k / 10.0;
					double num5 = num * (array[j] + num4 * (src[j] - array2[j - 1])) + (1.0 - num) * array2[j - 1];
					double value = src[j] - num5;
					if (Math.Abs(value) < num3)
					{
						num3 = Math.Abs(value);
						num2 = num4;
					}
				}
				array2[j] = num * (array[j] + num2 * (src[j] - array2[j - 1])) + (1.0 - num) * array2[j - 1];
			}
			return array2;
		}

		// Token: 0x17000464 RID: 1124
		public IContext Context
		{
			// Token: 0x06000D84 RID: 3460 RVA: 0x0003B340 File Offset: 0x00039540
			get;
			// Token: 0x06000D85 RID: 3461 RVA: 0x0003B348 File Offset: 0x00039548
			set;
		}

		// Token: 0x17000463 RID: 1123
		[HandlerParameter(true, "50", Min = "1", Max = "100", Step = "1")]
		public int GainLimit
		{
			// Token: 0x06000D80 RID: 3456 RVA: 0x0003B1BE File Offset: 0x000393BE
			get;
			// Token: 0x06000D81 RID: 3457 RVA: 0x0003B1C6 File Offset: 0x000393C6
			set;
		}

		// Token: 0x17000462 RID: 1122
		[HandlerParameter(true, "20", Min = "1", Max = "100", Step = "1")]
		public int Length
		{
			// Token: 0x06000D7E RID: 3454 RVA: 0x0003B1AD File Offset: 0x000393AD
			get;
			// Token: 0x06000D7F RID: 3455 RVA: 0x0003B1B5 File Offset: 0x000393B5
			set;
		}
	}
}
