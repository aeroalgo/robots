using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000154 RID: 340
	[HandlerCategory("vvAverages"), HandlerName("ADXVMA")]
	public class ADXVMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AAC RID: 2732 RVA: 0x0002C3EC File Offset: 0x0002A5EC
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("adxvma", new string[]
			{
				this.AdxVmaPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => ADXVMA.GenVMA(src, this.AdxVmaPeriod, this.Context));
		}

		// Token: 0x06000AAB RID: 2731 RVA: 0x0002C120 File Offset: 0x0002A320
		public static IList<double> GenVMA(IList<double> src, int _AdxVmaPeriod, IContext ctx)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[,] array2 = new double[count, 6];
			_AdxVmaPeriod = Math.Max(_AdxVmaPeriod, 1);
			for (int i = _AdxVmaPeriod; i < count; i++)
			{
				array2[i, 0] = src[i];
				double num = array2[i, 0] - array2[i - 1, 0];
				double num2 = 0.0;
				double num3 = 0.0;
				if (num > 0.0)
				{
					num2 = num;
				}
				else
				{
					num3 = -num;
				}
				array2[i, 1] = (((double)_AdxVmaPeriod - 1.0) * array2[i - 1, 1] + num2) / (double)_AdxVmaPeriod;
				array2[i, 2] = (((double)_AdxVmaPeriod - 1.0) * array2[i - 1, 2] + num3) / (double)_AdxVmaPeriod;
				double num4 = array2[i, 1] + array2[i, 2];
				double num5 = 0.0;
				double num6 = 0.0;
				if (num4 > 0.0)
				{
					num5 = array2[i, 1] / num4;
					num6 = array2[i, 2] / num4;
				}
				array2[i, 3] = (((double)_AdxVmaPeriod - 1.0) * array2[i - 1, 3] + num5) / (double)_AdxVmaPeriod;
				array2[i, 4] = (((double)_AdxVmaPeriod - 1.0) * array2[i - 1, 4] + num6) / (double)_AdxVmaPeriod;
				double num7 = 0.0;
				if (array2[i, 3] + array2[i, 4] > 0.0)
				{
					num7 = Math.Abs(array2[i, 3] - array2[i, 4]) / (array2[i, 3] + array2[i, 4]);
				}
				array2[i, 5] = (((double)_AdxVmaPeriod - 1.0) * array2[i - 1, 5] + num7) / (double)_AdxVmaPeriod;
				double num8 = Math.Max(array2[i, 5], array2[i - 1, 5]);
				double num9 = Math.Min(array2[i, 5], array2[i - 1, 5]);
				for (int j = 2; j < _AdxVmaPeriod; j++)
				{
					num8 = Math.Max(array2[i - j, 5], num8);
					num9 = Math.Min(array2[i - j, 5], num9);
				}
				double num10 = 0.0;
				if (num8 - num9 > 0.0)
				{
					num10 = (array2[i, 5] - num9) / (num8 - num9);
				}
				array[i] = (((double)_AdxVmaPeriod - num10) * array[i - 1] + num10 * array2[i, 0]) / (double)_AdxVmaPeriod;
			}
			return array;
		}

		// Token: 0x17000388 RID: 904
		[HandlerParameter(true, "10", Min = "5", Max = "30", Step = "1")]
		public int AdxVmaPeriod
		{
			// Token: 0x06000AA9 RID: 2729 RVA: 0x0002C10C File Offset: 0x0002A30C
			get;
			// Token: 0x06000AAA RID: 2730 RVA: 0x0002C114 File Offset: 0x0002A314
			set;
		}

		// Token: 0x17000389 RID: 905
		public IContext Context
		{
			// Token: 0x06000AAD RID: 2733 RVA: 0x0002C458 File Offset: 0x0002A658
			get;
			// Token: 0x06000AAE RID: 2734 RVA: 0x0002C460 File Offset: 0x0002A660
			set;
		}
	}
}
