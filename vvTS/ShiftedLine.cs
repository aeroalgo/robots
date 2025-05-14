using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000115 RID: 277
	[HandlerCategory("vvBands&Channels"), HandlerName("Сдвиг линии по вертикали")]
	public class ShiftedLine : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060007C7 RID: 1991 RVA: 0x00021D69 File Offset: 0x0001FF69
		public IList<double> Execute(IList<double> _list)
		{
			return ShiftedLine.GenShiftedLine(_list, this.Shift);
		}

		// Token: 0x060007C6 RID: 1990 RVA: 0x00021D1C File Offset: 0x0001FF1C
		public static IList<double> GenShiftedLine(IList<double> _list, double _shift)
		{
			IList<double> list = new double[_list.Count];
			for (int i = 0; i < _list.Count; i++)
			{
				list[i] = _list[i] + _list[i] / 100.0 * _shift;
			}
			return list;
		}

		// Token: 0x17000273 RID: 627
		[HandlerParameter(true, "0", Min = "-10", Max = "10", Step = "0.1")]
		public double Shift
		{
			// Token: 0x060007C4 RID: 1988 RVA: 0x00021D09 File Offset: 0x0001FF09
			get;
			// Token: 0x060007C5 RID: 1989 RVA: 0x00021D11 File Offset: 0x0001FF11
			set;
		}
	}
}
